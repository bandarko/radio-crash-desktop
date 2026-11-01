//! Optional optics pass. The original egui framebuffer is always painted first.
use super::policy;
use eframe::glow::{self, HasContext};
use std::time::Instant;

// glow 0.17 checks the GL 4.4 query-buffer entry point before using GL 3.3
// query reads. macOS 4.1 has core query reads but no EXT aliases. Load the
// actual core functions explicitly, checking both pointers before use.
#[derive(Clone, Copy)]
pub struct TimerApi {
    available: unsafe extern "system" fn(u32, u32, *mut u32),
    elapsed: unsafe extern "system" fn(u32, u32, *mut u64),
}
impl TimerApi {
    pub fn load(cc: &eframe::CreationContext<'_>) -> Option<Self> {
        let loader = cc.get_proc_address.as_ref()?;
        let a = loader(c"glGetQueryObjectuiv");
        let e = loader(c"glGetQueryObjectui64v");
        if [a, e]
            .iter()
            .any(|p| (*p as usize) <= 3 || (*p as usize) == usize::MAX)
        {
            return None;
        }
        // The GL context loader returns pointers to the named, documented ABI.
        Some(unsafe {
            Self {
                available: std::mem::transmute::<
                    *const core::ffi::c_void,
                    unsafe extern "system" fn(u32, u32, *mut u32),
                >(a),
                elapsed: std::mem::transmute::<
                    *const core::ffi::c_void,
                    unsafe extern "system" fn(u32, u32, *mut u64),
                >(e),
            }
        })
    }
    fn result(self, query: glow::Query) -> Option<f32> {
        unsafe {
            let mut ready = 0;
            (self.available)(query.0.get(), glow::QUERY_RESULT_AVAILABLE, &mut ready);
            if ready == 0 {
                return None;
            }
            let mut ns = 0;
            (self.elapsed)(query.0.get(), glow::QUERY_RESULT, &mut ns);
            Some(ns as f32 / 1_000_000.)
        }
    }
}
struct Capture {
    raw: glow::Texture,
    width: i32,
    height: i32,
    bloom: [glow::Texture; 2],
    fbos: [glow::Framebuffer; 2],
    small: [i32; 2],
}
struct Timer {
    query: glow::Query,
    pending: bool,
    at: Instant,
    kind: usize,
}
pub struct Renderer {
    program: glow::Program,
    vao: glow::VertexArray,
    source: Option<glow::UniformLocation>,
    bloom: Option<glow::UniformLocation>,
    stage: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    brightness: Option<glow::UniformLocation>,
    signal: Option<glow::UniformLocation>,
    volume: Option<glow::UniformLocation>,
    captures: [Option<Capture>; 2],
    timers: Vec<Timer>,
    pub gpu_ms: Option<f32>,
    pub completed: u64,
    performance: policy::Performance,
    timer_api: TimerApi,
    pass_ms: [Option<f32>; 2],
}
impl Renderer {
    pub fn new(gl: &glow::Context, timer_api: TimerApi) -> Result<Self, String> {
        // All handles created before a possible failure are explicitly released.
        unsafe {
            let program = gl.create_program()?;
            let mut shaders = Vec::new();
            let result = (|| {
                for (kind, source) in [
                    (glow::VERTEX_SHADER, include_str!("optics.vert")),
                    (glow::FRAGMENT_SHADER, include_str!("optics.frag")),
                ] {
                    let shader = gl.create_shader(kind)?;
                    shaders.push(shader);
                    gl.shader_source(shader, source);
                    gl.compile_shader(shader);
                    if !gl.get_shader_compile_status(shader) {
                        return Err(gl.get_shader_info_log(shader));
                    }
                    gl.attach_shader(program, shader);
                }
                gl.link_program(program);
                if !gl.get_program_link_status(program) {
                    return Err(gl.get_program_info_log(program));
                }
                gl.create_vertex_array()
            })();
            for shader in shaders {
                if gl.get_shader_compile_status(shader) {
                    gl.detach_shader(program, shader);
                }
                gl.delete_shader(shader);
            }
            let vao = match result {
                Ok(v) => v,
                Err(e) => {
                    gl.delete_program(program);
                    return Err(e);
                }
            };
            let mut timers = Vec::new();
            for _ in 0..4 {
                match gl.create_query() {
                    Ok(query) => timers.push(Timer {
                        query,
                        pending: false,
                        at: Instant::now(),
                        kind: 0,
                    }),
                    Err(e) => {
                        for timer in timers {
                            gl.delete_query(timer.query);
                        }
                        gl.delete_vertex_array(vao);
                        gl.delete_program(program);
                        return Err(e);
                    }
                }
            }
            let uniform = |name| gl.get_uniform_location(program, name);
            Ok(Self {
                program,
                vao,
                source: uniform("source_image"),
                bloom: uniform("bloom_image"),
                stage: uniform("stage"),
                resolution: uniform("resolution"),
                brightness: uniform("brightness"),
                signal: uniform("signal_light"),
                volume: uniform("volume"),
                captures: [None, None],
                timers,
                gpu_ms: None,
                completed: 0,
                performance: policy::Performance::default(),
                timer_api,
                pass_ms: [None, None],
            })
        }
    }
    pub fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            for capture in &mut self.captures {
                if let Some(c) = capture.take() {
                    c.destroy(gl);
                }
            }
            for timer in self.timers.drain(..) {
                gl.delete_query(timer.query);
            }
            gl.delete_vertex_array(self.vao);
            gl.delete_program(self.program);
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        gl: &glow::Context,
        target: Option<glow::Framebuffer>,
        viewport: [i32; 4],
        glass: bool,
        brightness: f32,
        signal: f32,
        volume: f32,
        max_texture: i32,
    ) -> Result<(), String> {
        let [x, y, width, height] = viewport;
        if !policy::within_budget(width.max(0) as u32, height.max(0) as u32, max_texture) {
            return Err(
                "Window exceeds the enhanced graphics memory budget. Original appearance retained."
                    .into(),
            );
        }
        // Timer queries are read only when ready, never synchronizing the CPU with the GPU.
        for timer in &mut self.timers {
            if timer.pending
                && let Some(ms) = self.timer_api.result(timer.query)
            {
                timer.pending = false;
                self.pass_ms[timer.kind] =
                    Some(self.pass_ms[timer.kind].map_or(ms, |old| old * 0.9 + ms * 0.1));
                self.gpu_ms = Some(self.pass_ms.iter().flatten().sum());
                let combined = ms + self.pass_ms[0].unwrap_or(0.);
                if timer.kind == 1 && self.performance.observe(combined) {
                    return Err(
                        "GPU effects exceeded the time budget. Original appearance retained."
                            .into(),
                    );
                }
            } else if timer.pending && timer.at.elapsed().as_secs() > 3 {
                return Err("GPU timing stalled. Original appearance retained.".into());
            }
        }
        let index = usize::from(!glass);
        let mut scissor = [0; 4];
        unsafe {
            gl.get_parameter_i32_slice(glow::SCISSOR_BOX, &mut scissor);
        }
        let scissor_enabled = unsafe { gl.is_enabled(glow::SCISSOR_TEST) };
        let result = (|| {
            unsafe {
                if self.captures[index]
                    .as_ref()
                    .is_none_or(|c| c.width != width || c.height != height)
                {
                    if let Some(c) = self.captures[index].take() {
                        c.destroy(gl);
                    }
                    self.captures[index] = Some(Capture::new(gl, width, height, glass)?);
                }
                gl.bind_framebuffer(glow::FRAMEBUFFER, target);
                let c = self.captures[index].as_ref().unwrap();
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(c.raw));
                gl.copy_tex_sub_image_2d(glow::TEXTURE_2D, 0, 0, 0, x, y, width, height);
                let error = gl.get_error();
                if error != glow::NO_ERROR {
                    return Err(format!(
                        "Framebuffer capture failed ({error:#x}). Original appearance retained."
                    ));
                }
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(c.raw));
                gl.active_texture(glow::TEXTURE0);
                gl.disable(glow::SCISSOR_TEST);
                gl.disable(glow::BLEND);
                gl.disable(glow::DEPTH_TEST);
                gl.disable(glow::CULL_FACE);
                gl.use_program(Some(self.program));
                gl.bind_vertex_array(Some(self.vao));
                gl.uniform_1_i32(self.source.as_ref(), 0);
                gl.uniform_1_i32(self.bloom.as_ref(), 1);
                gl.uniform_1_f32(self.brightness.as_ref(), brightness);
                gl.uniform_1_f32(self.signal.as_ref(), signal);
                gl.uniform_1_f32(self.volume.as_ref(), volume);
                let timer = self.timers.iter_mut().find(|t| !t.pending);
                let query = timer.map(|timer| {
                    timer.pending = true;
                    timer.kind = index;
                    timer.at = Instant::now();
                    gl.begin_query(glow::TIME_ELAPSED, timer.query);
                    timer.query
                });
                if glass {
                    gl.viewport(0, 0, c.small[0], c.small[1]);
                    gl.uniform_2_f32(
                        self.resolution.as_ref(),
                        c.small[0] as f32,
                        c.small[1] as f32,
                    );
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(c.fbos[0]));
                    gl.uniform_1_i32(self.stage.as_ref(), 1);
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(c.fbos[1]));
                    gl.bind_texture(glow::TEXTURE_2D, Some(c.bloom[0]));
                    gl.uniform_1_i32(self.stage.as_ref(), 2);
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(c.fbos[0]));
                    gl.bind_texture(glow::TEXTURE_2D, Some(c.bloom[1]));
                    gl.uniform_1_i32(self.stage.as_ref(), 3);
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    gl.active_texture(glow::TEXTURE1);
                    gl.bind_texture(glow::TEXTURE_2D, Some(c.bloom[0]));
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(glow::TEXTURE_2D, Some(c.raw));
                }
                gl.bind_framebuffer(glow::FRAMEBUFFER, target);
                gl.viewport(x, y, width, height);
                if scissor_enabled {
                    gl.enable(glow::SCISSOR_TEST);
                    gl.scissor(scissor[0], scissor[1], scissor[2], scissor[3]);
                }
                gl.uniform_2_f32(self.resolution.as_ref(), width as f32, height as f32);
                gl.uniform_1_i32(self.stage.as_ref(), if glass { 4 } else { 0 });
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                if query.is_some() {
                    gl.end_query(glow::TIME_ELAPSED);
                }
                let error = gl.get_error();
                if error != glow::NO_ERROR {
                    return Err(format!(
                        "GPU optics failed ({error:#x}). Original appearance retained."
                    ));
                }
                self.completed += 1;
            }
            Ok(())
        })();
        // egui restores its own shader/blending/viewport, but not this framebuffer.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, target);
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.active_texture(glow::TEXTURE0);
            gl.scissor(scissor[0], scissor[1], scissor[2], scissor[3]);
            if scissor_enabled {
                gl.enable(glow::SCISSOR_TEST);
            } else {
                gl.disable(glow::SCISSOR_TEST);
            }
        }
        result
    }
}
impl Capture {
    fn new(gl: &glow::Context, width: i32, height: i32, glass: bool) -> Result<Self, String> {
        unsafe {
            let mut textures = Vec::new();
            let mut fbos = Vec::new();
            let result = (|| {
                for _ in 0..3 {
                    textures.push(gl.create_texture()?);
                }
                for _ in 0..2 {
                    fbos.push(gl.create_framebuffer()?);
                }
                let small = if glass {
                    [(width / 4).max(1), (height / 4).max(1)]
                } else {
                    [1, 1]
                };
                for (i, &texture) in textures.iter().enumerate() {
                    gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                    for parameter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            parameter,
                            if i == 0 { glow::NEAREST } else { glow::LINEAR } as i32,
                        );
                    }
                    for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T] {
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            parameter,
                            glow::CLAMP_TO_EDGE as i32,
                        );
                    }
                    let (w, h, format, kind) = if i == 0 {
                        (width, height, glow::RGBA8, glow::UNSIGNED_BYTE)
                    } else {
                        (small[0], small[1], glow::RGBA16F, glow::FLOAT)
                    };
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        format as i32,
                        w,
                        h,
                        0,
                        glow::RGBA,
                        kind,
                        glow::PixelUnpackData::Slice(None),
                    );
                    if i > 0 {
                        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbos[i - 1]));
                        gl.framebuffer_texture_2d(
                            glow::FRAMEBUFFER,
                            glow::COLOR_ATTACHMENT0,
                            glow::TEXTURE_2D,
                            Some(texture),
                            0,
                        );
                        if gl.check_framebuffer_status(glow::FRAMEBUFFER)
                            != glow::FRAMEBUFFER_COMPLETE
                        {
                            return Err("Floating-point optics framebuffer is unsupported.".into());
                        }
                    }
                }
                if gl.get_error() != glow::NO_ERROR {
                    return Err("GPU optics allocation failed.".into());
                }
                Ok(small)
            })();
            match result {
                Ok(small) => Ok(Self {
                    raw: textures[0],
                    width,
                    height,
                    bloom: [textures[1], textures[2]],
                    fbos: [fbos[0], fbos[1]],
                    small,
                }),
                Err(e) => {
                    for texture in textures {
                        gl.delete_texture(texture);
                    }
                    for fbo in fbos {
                        gl.delete_framebuffer(fbo);
                    }
                    Err(e)
                }
            }
        }
    }
    fn destroy(self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.raw);
            for t in self.bloom {
                gl.delete_texture(t);
            }
            for f in self.fbos {
                gl.delete_framebuffer(f);
            }
        }
    }
}
