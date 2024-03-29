use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BYTE, DWORD, FALSE};
use winapi::shared::ntdef::BOOLEAN;
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
use winapi::um::winioctl::{IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_PROPERTY_QUERY, StorageDeviceProperty, PropertyStandardQuery};
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE};

// https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-storage_device_descriptor#syntax

#[repr(C)]
#[derive(Debug)]
struct StorageDeviceDescriptor
{
    version: DWORD,
    size: DWORD,
    device_type: BYTE,
    device_type_modifier: BYTE,
    removeable_media: BOOLEAN,
    command_queuing: BOOLEAN,
    vendor_id_offset: DWORD,
    product_id_offset: DWORD,
    product_revision_offset: DWORD,
    seial_number_offset: DWORD, // what we care about!
    bus_type: DWORD,
    raw_properties_length: DWORD,
    raw_device_properties: [BYTE; 1]
}

fn retrieve_serial(drive: &str) -> std::io::Result<String>
{
    let mut dw_returned: DWORD = 0;
    let mut buffer: [u8; 1024] = [0; 1024];

    let drive_cstring = CString::new(drive).unwrap();

    let handle: *mut c_void = unsafe
    {
        // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
        CreateFileA(
            drive_cstring.as_ptr(), // pointer to drive c string
            0, // neither read nor write permission
            FILE_SHARE_READ | FILE_SHARE_WRITE, // request read and write access after getting handle
            null_mut(), // we don't care about security
            OPEN_EXISTING, // only create handle if drive exists
            FILE_FLAG_BACKUP_SEMANTICS, // overrides file security
            null_mut() // i'm not sure what this does but I know we don't need it
        )
    };

    if handle == INVALID_HANDLE_VALUE
    {
        return Err(std::io::Error::last_os_error());
    }

    let mut query: STORAGE_PROPERTY_QUERY = STORAGE_PROPERTY_QUERY
    {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0; 1]
    };

    let ioctl_result = unsafe
    {
        winapi::um::ioapiset::DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            &mut query as *mut _ as *mut c_void,
            size_of::<STORAGE_PROPERTY_QUERY>() as DWORD,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as DWORD,
            &mut dw_returned,
            null_mut()
        )
    };

    if ioctl_result == FALSE
    {
        return Err(std::io::Error::last_os_error());
    }

    let descriptor: StorageDeviceDescriptor = unsafe{ std::ptr::read(buffer.as_ptr() as *const _) };

    let serial_number = unsafe{ CStr::from_ptr(buffer.as_ptr().offset(descriptor.seial_number_offset as isize) as *const _).to_str().unwrap() };

    Ok(serial_number.to_string())
}
fn main()
{
    for n in 0..9
    {
        let drive = format!("\\\\.\\PhysicalDrive{}", n);
        match retrieve_serial(&drive)
        {
            Ok(serial) => println!("SerialNumber\n{}", serial),
            Err(e) =>
            {
                if !e.to_string().contains("The system cannot find the file specified. (os error 2)")
                {
                    println!("Error: {}", e)
                }
            }
        }
    }

    let mut buf = "".to_string();
    let _ = std::io::stdin().read_line(&mut buf);
}