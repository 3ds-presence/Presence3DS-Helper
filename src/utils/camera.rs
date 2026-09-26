use std::ptr::null_mut;

use ctru::error::ResultCode;
use ctru::services::cam::{
    Cam, Camera as _, FrameRate, OutputFormat, PhotoMode, Trimming, ViewSize,
};
use ctru_sys::{
    CAMU_Activate, CAMU_ClearBuffer, CAMU_GetBufferErrorInterruptEvent, CAMU_GetMaxBytes,
    CAMU_SetReceiving, CAMU_SetTransferBytes, CAMU_StartCapture, CAMU_StopCapture, GFX_LEFT,
    GFX_TOP, Handle, SELECT_NONE, gfxFlushBuffers, gfxGetFramebuffer,
    gfxScreenSwapBuffers, svcCloseHandle,
    svcWaitSynchronizationN,
};

pub struct Camera {
    _cam: Cam,
    frame: Vec<u8>,
    width: usize,
    height: usize,
    port: u32,
    select: u32,
    transfer_unit: i16,
    receive_event: Handle,
    error_event: Handle,
    capturing: bool,
}

impl Camera {
    pub fn new() -> ctru::Result<Self> {
        let mut cam = Cam::new()?;
        let camera = &mut cam.outer_right_cam;

        for setting in [
            camera.set_view_size(ViewSize::TopLCD),
            camera.set_output_format(OutputFormat::Rgb565),
            camera.set_photo_mode(PhotoMode::Normal),
            camera.set_frame_rate(FrameRate::Fps30To10),
            camera.set_noise_filter(true),
            camera.set_auto_exposure(true),
            camera.set_trimming(Trimming::Off),
        ] {
            let _ = setting;
        }

        let (width, height) = camera.final_view_size();
        let (port, select) = (camera.port_as_raw(), camera.camera_as_raw());
        let frame = vec![0; camera.final_byte_length()];

        let mut unit = 0;
        unsafe {
            ResultCode(CAMU_GetMaxBytes(&raw mut unit, width, height))?;
            ResultCode(CAMU_SetTransferBytes(port, unit, width, height))?;
        }

        let mut camera = Self {
            _cam: cam,
            frame,
            width: usize::try_from(width).unwrap_or_default(),
            height: usize::try_from(height).unwrap_or_default(),
            port,
            select,
            transfer_unit: i16::try_from(unit).unwrap_or(i16::MAX),
            receive_event: 0,
            error_event: 0,
            capturing: false,
        };
        camera.start()?;

        Ok(camera)
    }

    fn start(&mut self) -> ctru::Result<()> {
        unsafe {
            ResultCode(CAMU_Activate(self.select))?;
            ResultCode(CAMU_GetBufferErrorInterruptEvent(
                &raw mut self.error_event,
                self.port,
            ))?;
            ResultCode(CAMU_ClearBuffer(self.port))?;
        }

        // The first transfer must be armed before capture starts.
        self.arm()?;
        unsafe {
            ResultCode(CAMU_StartCapture(self.port))?;
        }
        self.capturing = true;
        Ok(())
    }

    pub fn poll(&mut self) -> bool {
        if !self.capturing {
            self.capturing = unsafe { ResultCode(CAMU_StartCapture(self.port)).0 } >= 0;
            if !self.capturing {
                return false;
            }
        }

        if self.arm().is_err() {
            return false;
        }

        let mut index = -1;
        let events = [self.error_event, self.receive_event];
        unsafe {
            let _ = svcWaitSynchronizationN(
                &raw mut index,
                events.as_ptr(),
                i32::try_from(events.len()).unwrap_or_default(),
                false,
                100_000_000, // 1 second
            );
        }

        match index {
            // An overrun stops the capture, so clear it and restart.
            0 => {
                self.close_receive_event();
                unsafe {
                    let _ = CAMU_ClearBuffer(self.port);
                }
                let _ = self.arm();
                self.capturing = unsafe { ResultCode(CAMU_StartCapture(self.port)).0 } >= 0;
                false
            }
            1 => {
                let received = self.receive_event;
                self.receive_event = 0;
                let result = self.arm();
                unsafe {
                    let _ = svcCloseHandle(received);
                }
                result.is_ok()
            }
            _ => false,
        }
    }

    pub fn frame(&self) -> &[u8] {
        &self.frame
    }

    pub const fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn close_receive_event(&mut self) {
        if self.receive_event != 0 {
            unsafe {
                let _ = svcCloseHandle(self.receive_event);
            }
            self.receive_event = 0;
        }
    }

    fn arm(&mut self) -> ctru::Result<()> {
        if self.receive_event != 0 {
            return Ok(());
        }

        unsafe {
            ResultCode(CAMU_SetReceiving(
                &raw mut self.receive_event,
                self.frame.as_mut_ptr().cast(),
                self.port,
                u32::try_from(self.frame.len()).unwrap_or_default(),
                self.transfer_unit,
            ))?;
        }
        Ok(())
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.close_receive_event();

        unsafe {
            if self.error_event != 0 {
                let _ = svcCloseHandle(self.error_event);
            }
            let _ = CAMU_StopCapture(self.port);
            let _ = CAMU_ClearBuffer(self.port);
            let _ = CAMU_Activate(SELECT_NONE.into());
        }
    }
}

pub fn blit_to_top_screen(frame: &[u8], width: usize, height: usize) {
    if frame.len() < width * height * 2 {
        return;
    }

    let framebuffer = unsafe { gfxGetFramebuffer(GFX_TOP, GFX_LEFT, null_mut(), null_mut()) };
    if framebuffer.is_null() {
        return;
    }

    // Camera frame is horizontal RGB565, the screen expects rotated data:
    // rotate 90 degrees while copying, pixel by pixel in u16.
    #[allow(clippy::cast_ptr_alignment)]
    let src = frame.as_ptr().cast::<u16>();
    
    #[allow(clippy::cast_ptr_alignment)]
    let dst = framebuffer.cast::<u16>();
    
    for row in 0..height {
        for column in 0..width {
            let pixel = unsafe { *src.add(row * width + column) };
            let address = column * height + (height - 1 - row);
            unsafe {
                *dst.add(address) = pixel;
            }
        }
    }

    // Flush the CPU cache, then swap: with double buffering the image
    // only becomes visible on the next vblank, which avoids tearing.
    unsafe {
        gfxFlushBuffers();
        gfxScreenSwapBuffers(GFX_TOP, false);
    }
}
