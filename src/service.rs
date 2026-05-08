#[cfg(windows)]
mod windows_impl {
    use crate::config::CliArgs;
    use anyhow::{Context, Result};
    use std::ffi::{OsStr, c_void};
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Builder;
    use tokio_util::sync::CancellationToken;
    use windows_sys::Win32::Foundation::{ERROR_FAILED_SERVICE_CONTROLLER_CONNECT, GetLastError};
    use windows_sys::Win32::System::Services::{
        CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
        OpenServiceW, QueryServiceStatus, RegisterServiceCtrlHandlerExW, SC_HANDLE,
        SC_MANAGER_CONNECT, SC_MANAGER_CREATE_SERVICE, SERVICE_ACCEPT_SHUTDOWN,
        SERVICE_ACCEPT_STOP, SERVICE_ALL_ACCESS, SERVICE_AUTO_START, SERVICE_CONTROL_INTERROGATE,
        SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_STOP, SERVICE_ERROR_NORMAL, SERVICE_QUERY_STATUS,
        SERVICE_RUNNING, SERVICE_START, SERVICE_START_PENDING, SERVICE_STATUS,
        SERVICE_STATUS_HANDLE, SERVICE_STOP, SERVICE_STOPPED, SERVICE_TABLE_ENTRYW,
        SERVICE_WIN32_OWN_PROCESS, SetServiceStatus, StartServiceCtrlDispatcherW, StartServiceW,
    };

    pub const SERVICE_NAME: &str = "rs-admin";
    pub const SERVICE_DISPLAY_NAME: &str = "rs-admin";

    static SERVICE_SHUTDOWN: OnceLock<CancellationToken> = OnceLock::new();

    pub fn install(args: &CliArgs) -> Result<()> {
        let scm = open_sc_manager(SC_MANAGER_CONNECT | SC_MANAGER_CREATE_SERVICE)?;
        let exe = std::env::current_exe().context("获取当前可执行文件路径失败")?;
        let mut binary = quote_path(&exe);
        for arg in args.service_launch_arguments() {
            binary.push(' ');
            binary.push_str(&escape_service_arg(&arg.to_string_lossy()));
        }

        let name = to_wide(SERVICE_NAME);
        let display = to_wide(SERVICE_DISPLAY_NAME);
        let binary_w = to_wide(&binary);

        let service = unsafe {
            CreateServiceW(
                scm,
                name.as_ptr(),
                display.as_ptr(),
                SERVICE_ALL_ACCESS,
                SERVICE_WIN32_OWN_PROCESS,
                SERVICE_AUTO_START,
                SERVICE_ERROR_NORMAL,
                binary_w.as_ptr(),
                null(),
                null_mut(),
                null(),
                null(),
                null(),
            )
        };
        if service.is_null() {
            unsafe { CloseServiceHandle(scm) };
            return Err(std::io::Error::last_os_error()).context("创建服务失败");
        }
        unsafe {
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
        }
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        let scm = open_sc_manager(SC_MANAGER_CONNECT)?;
        let service = open_service(scm, SERVICE_NAME, SERVICE_ALL_ACCESS)?;
        unsafe {
            if DeleteService(service) == 0 {
                let err = std::io::Error::last_os_error();
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(err).context("删除服务失败");
            }
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
        }
        Ok(())
    }

    pub fn start() -> Result<()> {
        let scm = open_sc_manager(SC_MANAGER_CONNECT)?;
        let service = open_service(scm, SERVICE_NAME, SERVICE_START | SERVICE_QUERY_STATUS)?;
        unsafe {
            if StartServiceW(service, 0, null()) == 0 {
                let err = std::io::Error::last_os_error();
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(err).context("启动服务失败");
            }
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
        }
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let scm = open_sc_manager(SC_MANAGER_CONNECT)?;
        let service = open_service(scm, SERVICE_NAME, SERVICE_STOP | SERVICE_QUERY_STATUS)?;
        unsafe {
            let mut status: SERVICE_STATUS = std::mem::zeroed();
            if ControlService(service, SERVICE_CONTROL_STOP, &mut status) == 0 {
                let err = std::io::Error::last_os_error();
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(err).context("停止服务失败");
            }
            wait_until_stopped(service)?;
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
        }
        Ok(())
    }

    pub fn try_run_dispatcher() -> Result<bool> {
        let name = to_wide(SERVICE_NAME);
        let mut table = [
            SERVICE_TABLE_ENTRYW {
                lpServiceName: name.as_ptr() as *mut u16,
                lpServiceProc: Some(service_main),
            },
            SERVICE_TABLE_ENTRYW {
                lpServiceName: null_mut(),
                lpServiceProc: None,
            },
        ];
        let ok = unsafe { StartServiceCtrlDispatcherW(table.as_mut_ptr()) };
        if ok != 0 {
            return Ok(true);
        }
        let err = unsafe { GetLastError() };
        if err == ERROR_FAILED_SERVICE_CONTROLLER_CONNECT {
            return Ok(false);
        }
        Err(std::io::Error::last_os_error()).context("连接 Windows Service 控制器失败")
    }

    unsafe extern "system" fn service_main(_argc: u32, _argv: *mut *mut u16) {
        if let Err(e) = service_main_inner() {
            tracing::error!("Windows Service exited with error: {e:#}");
        }
    }

    fn service_main_inner() -> Result<()> {
        let args = CliArgs::parse();
        let shutdown = CancellationToken::new();
        let _ = SERVICE_SHUTDOWN.set(shutdown.clone());

        let status_handle = register_service_handler()?;
        set_status(status_handle, SERVICE_START_PENDING)?;

        let rt = Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("创建 Tokio runtime 失败")?;

        set_status(status_handle, SERVICE_RUNNING)?;
        let run_result = rt.block_on(async move { crate::run_with_shutdown(args, shutdown).await });

        set_status(status_handle, SERVICE_STOPPED)?;
        run_result
    }

    unsafe extern "system" fn service_handler(
        control: u32,
        _event_type: u32,
        _event_data: *mut c_void,
        _context: *mut c_void,
    ) -> u32 {
        match control {
            SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
                if let Some(token) = SERVICE_SHUTDOWN.get() {
                    token.cancel();
                }
                0
            }
            SERVICE_CONTROL_INTERROGATE => 0,
            _ => 0,
        }
    }

    fn register_service_handler() -> Result<SERVICE_STATUS_HANDLE> {
        let name = to_wide(SERVICE_NAME);
        let handle = unsafe {
            RegisterServiceCtrlHandlerExW(name.as_ptr(), Some(service_handler), null_mut())
        };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error()).context("注册服务控制处理器失败");
        }
        Ok(handle)
    }

    fn set_status(handle: SERVICE_STATUS_HANDLE, state: u32) -> Result<()> {
        let status = SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: state,
            dwControlsAccepted: if state == SERVICE_RUNNING {
                SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN
            } else {
                0
            },
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };
        let ok = unsafe { SetServiceStatus(handle, &status) };
        if ok == 0 {
            return Err(std::io::Error::last_os_error()).context("设置服务状态失败");
        }
        Ok(())
    }

    fn wait_until_stopped(service: SC_HANDLE) -> Result<()> {
        for _ in 0..60 {
            let mut status: SERVICE_STATUS = unsafe { std::mem::zeroed() };
            let ok = unsafe { QueryServiceStatus(service, &mut status) };
            if ok == 0 {
                return Err(std::io::Error::last_os_error()).context("查询服务状态失败");
            }
            if status.dwCurrentState == SERVICE_STOPPED {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(500));
        }
        Ok(())
    }

    fn open_sc_manager(access: u32) -> Result<SC_HANDLE> {
        let scm = unsafe { OpenSCManagerW(null(), null(), access) };
        if scm.is_null() {
            return Err(std::io::Error::last_os_error()).context("连接服务管理器失败");
        }
        Ok(scm)
    }

    fn open_service(scm: SC_HANDLE, name: &str, access: u32) -> Result<SC_HANDLE> {
        let wide = to_wide(name);
        let service = unsafe { OpenServiceW(scm, wide.as_ptr(), access) };
        if service.is_null() {
            return Err(std::io::Error::last_os_error()).context("打开服务失败");
        }
        Ok(service)
    }

    fn to_wide(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn quote_path(path: &std::path::Path) -> String {
        format!("\"{}\"", path.to_string_lossy())
    }

    fn escape_service_arg(value: &str) -> String {
        if value.is_empty() {
            return "\"\"".to_string();
        }
        if value.contains(' ') || value.contains('\t') || value.contains('"') {
            let escaped = value.replace('"', "\\\"");
            format!("\"{}\"", escaped)
        } else {
            value.to_string()
        }
    }
}

#[cfg(not(windows))]
mod windows_impl {
    use crate::config::CliArgs;
    use anyhow::{Result, bail};

    pub fn install(_args: &CliArgs) -> Result<()> {
        bail!("Windows Service 仅在 Windows 上可用");
    }

    pub fn uninstall() -> Result<()> {
        bail!("Windows Service 仅在 Windows 上可用");
    }

    pub fn start() -> Result<()> {
        bail!("Windows Service 仅在 Windows 上可用");
    }

    pub fn stop() -> Result<()> {
        bail!("Windows Service 仅在 Windows 上可用");
    }

    pub fn try_run_dispatcher() -> Result<bool> {
        Ok(false)
    }
}

pub use windows_impl::*;
