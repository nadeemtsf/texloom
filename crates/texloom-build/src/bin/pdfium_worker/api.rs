//! Small audited ABI surface from the pinned PDFium 126 development binding.
//! No PDFium code is loaded into the application process.
use sha2::{Digest, Sha256};
use std::{
    ffi::{c_char, c_void},
    fs::File,
    io::Read,
    marker::PhantomData,
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt},
    path::Path,
};
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HMODULE},
    System::LibraryLoader::{
        GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
    },
};
pub type Handle = *mut c_void;
#[repr(C)]
#[derive(Default)]
pub struct PageRect { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 }
pub type Result<T> = std::result::Result<T, String>;
pub use texloom_build::inspection_worker::DLL_SHA256;
struct Library {
    module: HMODULE,
    _file: File,
}
impl Library {
    fn open(path: &Path) -> Result<Self> {
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let mut file = File::options()
            .read(true)
            .share_mode(1)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(30_000_001)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 30_000_000 || format!("{:x}", Sha256::digest(&bytes)) != DLL_SHA256 {
            return Err("Independent inspector DLL checksum mismatch".into());
        }
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Absolute path and explicit search flags avoid loading dependencies
        // from the compiler/project working directory.
        let module = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            )
        };
        if module.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(Self {
            module,
            _file: file,
        })
    }
    fn symbol<T: Copy>(&self, name: &[u8]) -> Result<T> {
        let function = unsafe { GetProcAddress(self.module, name.as_ptr()) }.ok_or_else(|| {
            format!(
                "Missing pinned PDFium function: {}",
                String::from_utf8_lossy(name)
            )
        })?;
        let pointer = function as *const ();
        if std::mem::size_of::<T>() != std::mem::size_of_val(&pointer) {
            return Err("Invalid PDFium ABI width".into());
        }
        // T is instantiated only by the exact function signatures below.
        Ok(unsafe { std::mem::transmute_copy(&pointer) })
    }
}
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.module);
        }
    }
}
macro_rules! api {
    ($($name:ident($($argument:ty),*) -> $result:ty;)+)=>{
        #[allow(non_snake_case)]
        pub struct Pdfium {$(pub $name:pub_fn!($($argument),* => $result),)+_library:Library}
        impl Pdfium {
            pub fn load(path:&Path)->Result<Self>{
                let library=Library::open(path)?;
                let api=Self{$($name:library.symbol(concat!(stringify!($name),"\0").as_bytes())?,)+_library:library};
                unsafe{(api.FPDF_InitLibrary)()};Ok(api)
            }
        }
    }
}
macro_rules! pub_fn {($($argument:ty),* => $result:ty)=>{unsafe extern "system" fn($($argument),*)->$result};}
// These signatures match the pinned Windows x64 C ABI; handles stay in this
// process and never cross IPC. The DLL hash is checked before symbol loading.
api! {
    FPDF_InitLibrary()->();
    FPDF_DestroyLibrary()->();
    FPDF_LoadMemDocument64(*const c_void,usize,*const c_char)->Handle;
    FPDF_CloseDocument(Handle)->();
    FPDF_GetPageCount(Handle)->i32;
    FPDF_LoadPage(Handle,i32)->Handle;
    FPDF_ClosePage(Handle)->();
    FPDF_GetPageWidthF(Handle)->f32;
    FPDF_GetPageHeightF(Handle)->f32;
    FPDF_GetPageBoundingBox(Handle,*mut PageRect)->i32;
    FPDFPage_GetRotation(Handle)->i32;
    FPDFPage_GetAnnotCount(Handle)->i32;
    FPDFText_LoadPage(Handle)->Handle;
    FPDFText_ClosePage(Handle)->();
    FPDFText_CountChars(Handle)->i32;
    FPDFText_GetUnicode(Handle,i32)->u32;
    FPDFText_GetCharBox(Handle,i32,*mut f64,*mut f64,*mut f64,*mut f64)->i32;
    FPDFPage_CountObjects(Handle)->i32;
    FPDFPage_GetObject(Handle,i32)->Handle;
    FPDFPageObj_GetType(Handle)->i32;
    FPDFPageObj_CountMarks(Handle)->i32;
    FPDFPageObj_GetMark(Handle,u32)->Handle;
    FPDFPageObjMark_GetName(Handle,*mut c_void,u32,*mut u32)->i32;
    FPDFPageObj_GetBounds(Handle,*mut f32,*mut f32,*mut f32,*mut f32)->i32;
    FPDFBitmap_CreateEx(i32,i32,i32,*mut c_void,i32)->Handle;
    FPDFBitmap_Destroy(Handle)->();
    FPDFBitmap_FillRect(Handle,i32,i32,i32,i32,u32)->();
    FPDF_RenderPageBitmap(Handle,Handle,i32,i32,i32,i32,i32,i32)->();
    FPDFBitmap_GetStride(Handle)->i32;
    FPDFBitmap_GetBuffer(Handle)->Handle;
}
impl Drop for Pdfium {
    fn drop(&mut self) {
        unsafe {
            (self.FPDF_DestroyLibrary)();
        }
    }
}
pub struct Owned<'a> {
    pub raw: Handle,
    close: unsafe extern "system" fn(Handle),
    _api: PhantomData<&'a Pdfium>,
}
impl Pdfium {
    pub fn own(&self, raw: Handle, close: unsafe extern "system" fn(Handle)) -> Result<Owned<'_>> {
        if raw.is_null() {
            return Err("PDFium could not open this resource".into());
        }
        Ok(Owned {
            raw,
            close,
            _api: PhantomData,
        })
    }
}
impl Drop for Owned<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.close)(self.raw);
        }
    }
}
