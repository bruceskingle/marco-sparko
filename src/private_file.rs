use std::fs::File;
use std::io;
use std::path::Path;


#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use std::ptr::null_mut;


#[cfg(windows)]
use windows::Win32::Security::Authorization::*;
#[cfg(windows)]
use std::os::windows::io::FromRawHandle;

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

pub fn create_private_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    use std::fs::DirBuilder;

    let mut builder = DirBuilder::new();
#[cfg(unix)]
    builder.mode(0o700);
    
// #[cfg(windows)]
//     builder.set_permissions(0o700);
    match builder
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
}

#[cfg(windows)]
pub fn create_private_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    unsafe {
        let path_ref: &Path = path.as_ref(); // convert generic P to &Path

        // Convert path to wide string
        let wide: Vec<u16> = path_ref.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();

            
        // Create file first
        let handle = CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_NONE,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?;

        if handle == INVALID_HANDLE_VALUE {
            panic!("CreateFileW failed: {}", std::io::Error::last_os_error());
        }

        // Open process token
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        // Get user SID
        let mut len = 0u32;
        GetTokenInformation(token, TokenUser, None, 0, &mut len)?;
        let mut buffer = vec![0u8; len as usize];
        GetTokenInformation(token, TokenUser, Some(buffer.as_mut_ptr() as _), len, &mut len)?;
        let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);
        let user_sid = token_user.User.Sid;

        // Build EXPLICIT_ACCESS allowing only the current user
        let mut ea = EXPLICIT_ACCESS_W::default();
        ea.grfAccessPermissions = GENERIC_READ.0 | GENERIC_WRITE.0;
        ea.grfAccessMode = SET_ACCESS;
        ea.grfInheritance = NO_INHERITANCE;
        BuildTrusteeWithSidW(&mut ea.Trustee, user_sid);

        let ea_slice = [ea];
        let mut dacl: *mut ACL = null_mut();
        SetEntriesInAclW(Some(&ea_slice), None, &mut dacl);

        // Apply DACL only (do NOT pass owner)
        SetNamedSecurityInfoW(
            windows::core::PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,   // owner stays as-is
            None,
            Some(dacl),
            None,
        );

        println!("Created private file");

        // SAFETY: we own the handle and no one else will close it
        let file = File::from_raw_handle(handle.0 as _) ;

        Ok(file)
    }
}