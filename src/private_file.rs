use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::ptr::{null, null_mut};

#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Security::*;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows::Win32::System::Threading::*;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

#[cfg(unix)]
pub fn create_private_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    use std::fs::OpenOptions;

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    // Set file mode to 0o600 (read and write for owner only)
    options.mode(0o600);
    options.open(path)
}


#[cfg(unix)]
pub fn create_private_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    use std::fs::DirBuilder;

    match DirBuilder::new()
        .mode(0o700)
        .create(path)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    // let mut builder = DirBuilder::new();
    // builder.mode(0o700);
    // builder.create(path)
}

#[cfg(windows)]
pub fn create_private_file(path: &str) -> io::Result<File> {
    unsafe {
        /* ---------------------------------------------------------
         * 1. Get current user token
         * --------------------------------------------------------- */
        let token = {
            let mut token = HANDLE::default();
            if !OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut token,
            )
            .as_bool()
            {
                return Err(io::Error::last_os_error());
            }
            token
        };

        /* ---------------------------------------------------------
         * 2. Query size of TOKEN_USER
         * --------------------------------------------------------- */
        let mut len = 0u32;
        GetTokenInformation(
            token,
            TokenUser,
            None,
            0,
            &mut len,
        );

        let mut buffer = vec![0u8; len as usize];

        if !GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr() as _),
            len,
            &mut len,
        )
        .as_bool()
        {
            return Err(io::Error::last_os_error());
        }

        let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);
        let user_sid = token_user.User.Sid;

        /* ---------------------------------------------------------
         * 3. Create explicit access entry for user only
         * --------------------------------------------------------- */
        let mut ea = EXPLICIT_ACCESS_W::default();
        ea.grfAccessPermissions = GENERIC_READ | GENERIC_WRITE;
        ea.grfAccessMode = SET_ACCESS;
        ea.grfInheritance = NO_INHERITANCE;
        ea.Trustee.TrusteeForm = TRUSTEE_FORM(TRUSTEE_IS_SID);
        ea.Trustee.TrusteeType = TRUSTEE_TYPE(TRUSTEE_IS_USER);
        ea.Trustee.ptstrName = PWSTR(user_sid as _);

        /* ---------------------------------------------------------
         * 4. Create DACL
         * --------------------------------------------------------- */
        let mut dacl: *mut ACL = null_mut();
        let result = SetEntriesInAclW(
            1,
            &ea,
            None,
            &mut dacl,
        );

        if result != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(result.0 as i32));
        }

        /* ---------------------------------------------------------
         * 5. Create security descriptor
         * --------------------------------------------------------- */
        let mut sd = SECURITY_DESCRIPTOR::default();
        if !InitializeSecurityDescriptor(
            &mut sd,
            SECURITY_DESCRIPTOR_REVISION,
        )
        .as_bool()
        {
            return Err(io::Error::last_os_error());
        }

        if !SetSecurityDescriptorDacl(
            &mut sd,
            TRUE,
            dacl,
            FALSE,
        )
        .as_bool()
        {
            return Err(io::Error::last_os_error());
        }

        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: &mut sd as *mut _ as _,
            bInheritHandle: FALSE,
        };

        /* ---------------------------------------------------------
         * 6. Create file
         * --------------------------------------------------------- */
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let handle = CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_NONE,
            Some(&sa),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        CloseHandle(handle);
        LocalFree(dacl as _);

        Ok(())
    }
}
