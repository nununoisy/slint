// Copyright © SixtyFPS GmbH <info@sixtyfps.io>
// SPDX-License-Identifier: (GPL-3.0-only OR LicenseRef-SixtyFPS-commercial)

/*! This crate just expose the function used by the C++ integration */

use core::ffi::c_void;
use slint_backend_selector_internal::backend;
use slint_core_internal::window::ffi::WindowRcOpaque;
use slint_core_internal::window::WindowRc;

#[doc(hidden)]
#[cold]
pub fn use_modules() -> usize {
    #[cfg(feature = "slint-interpreter")]
    slint_interpreter::use_modules();
    slint_backend_selector_internal::use_modules();
    slint_core_internal::use_modules()
}

#[no_mangle]
pub unsafe extern "C" fn slint_windowrc_init(out: *mut WindowRcOpaque) {
    assert_eq!(core::mem::size_of::<WindowRc>(), core::mem::size_of::<WindowRcOpaque>());
    core::ptr::write(out as *mut WindowRc, crate::backend().create_window());
}

#[no_mangle]
pub unsafe extern "C" fn slint_run_event_loop() {
    crate::backend().run_event_loop(
        slint_core_internal::backend::EventLoopQuitBehavior::QuitOnLastWindowClosed,
    );
}

/// Will execute the given functor in the main thread
#[no_mangle]
pub unsafe extern "C" fn slint_post_event(
    event: extern "C" fn(user_data: *mut c_void),
    user_data: *mut c_void,
    drop_user_data: Option<extern "C" fn(*mut c_void)>,
) {
    struct UserData {
        user_data: *mut c_void,
        drop_user_data: Option<extern "C" fn(*mut c_void)>,
    }
    impl Drop for UserData {
        fn drop(&mut self) {
            if let Some(x) = self.drop_user_data {
                x(self.user_data)
            }
        }
    }
    unsafe impl Send for UserData {}
    let ud = UserData { user_data, drop_user_data };

    crate::backend().post_event(Box::new(move || {
        let ud = &ud;
        event(ud.user_data);
    }));
}

#[no_mangle]
pub unsafe extern "C" fn slint_quit_event_loop() {
    crate::backend().quit_event_loop();
}

#[no_mangle]
pub unsafe extern "C" fn slint_register_font_from_path(
    path: &slint_core_internal::SharedString,
    error_str: *mut slint_core_internal::SharedString,
) {
    core::ptr::write(
        error_str,
        match crate::backend().register_font_from_path(std::path::Path::new(path.as_str())) {
            Ok(()) => Default::default(),
            Err(err) => err.to_string().into(),
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn slint_register_font_from_data(
    data: slint_core_internal::slice::Slice<'static, u8>,
    error_str: *mut slint_core_internal::SharedString,
) {
    core::ptr::write(
        error_str,
        match crate::backend().register_font_from_memory(data.as_slice()) {
            Ok(()) => Default::default(),
            Err(err) => err.to_string().into(),
        },
    )
}

#[cfg(feature = "testing")]
#[no_mangle]
pub unsafe extern "C" fn slint_testing_init_backend() {
    slint_backend_testing_internal::init();
}
