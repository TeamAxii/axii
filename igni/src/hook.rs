use core::ptr::copy_nonoverlapping;
use windows::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};

pub unsafe fn copy_rw<T>(src: *const T, dst: *mut T, count: usize) {
    let size = count * size_of::<T>();
    let mut old_protect = Default::default();
    unsafe { VirtualProtect(src.cast(), size, PAGE_EXECUTE_READWRITE, &mut old_protect).unwrap() };
    unsafe { copy_nonoverlapping(src, dst, count) };
    unsafe { VirtualProtect(src.cast(), size, old_protect, &mut old_protect).unwrap() };
}

pub trait Hookable<F>: Copy {
    fn hook(self, function: F) {
        let ptr = self.as_u8_ptr();
        let func = self.thunk(function);
        dbg!(func);

        let bytes = {
            let mut jmp_bytes: [u8; 14] = [
                0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];
            jmp_bytes[6..14].copy_from_slice(&(func.0 as usize).to_le_bytes());
            jmp_bytes
        };
        println!("Bytes: {:x?}", bytes);

        unsafe { copy_rw(bytes.as_ptr(), ptr, bytes.len()) };
        println!("Placement done")
    }

    fn as_u8_ptr(self) -> *mut u8;

    fn thunk(self, function: F) -> (*const (), *mut F);
}

macro_rules! impl_hookable {
    ($(($($args:ident),*)),*) => {
        $(
            #[allow(non_snake_case)]
            impl<F, R, $($args),*> Hookable<F> for *const unsafe extern "C" fn($($args),*) -> R
            where
                F: FnMut($($args),*) {
                    fn as_u8_ptr(self) -> *mut u8 {
                        self as *mut u8
                    }

                    fn thunk(self, function: F) -> (*const (), *mut F) {
                        unsafe extern "C" fn ffi_thunk<F, $($args),*>(data: *mut F, $($args: $args),*)
                        where
                            F: FnMut($($args),*) {
                            (*(data as *mut F))($($args),*)
                        }

                        let data = Box::into_raw(Box::new(function));
                        (ffi_thunk::<F, $($args),*> as *const (), data)
                    }
                }

            #[allow(non_snake_case)]
            impl<F, R, $($args),*> Hookable<F> for *const unsafe extern "win64" fn($($args),*) -> R
            where
                F: FnMut($($args),*) {
                    fn as_u8_ptr(self) -> *mut u8 {
                        self as *mut u8
                    }

                    fn thunk(self, function: F) -> (*const (), *mut F) {
                        unsafe extern "win64" fn ffi_thunk<F, $($args),*>(data: *mut F, $($args: $args),*)
                        where
                            F: FnMut($($args),*) {
                            (*(data as *mut F))($($args),*)
                        }

                        let data = Box::into_raw(Box::new(function));
                        (ffi_thunk::<F, $($args),*> as *const (), data)
                    }
                }
        )*
    };
}

impl_hookable! {
    (),
    (A1),
    (A1, A2),
    (A1, A2, A3),
    (A1, A2, A3, A4)
}
