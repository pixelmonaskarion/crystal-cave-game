#[cfg(target_os = "android")] 
use winit::platform::android::activity::AndroidApp;

mod game;
mod instance;
mod runner;
mod cube;
mod light;
#[allow(unused)]
mod blur;
mod texture_types;
mod point_shadow;

include!(concat!(env!("OUT_DIR"), "/resources.rs"));

#[no_mangle]
#[cfg(target_os = "android")] 
fn android_main(app: AndroidApp) {
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));

    let event_loop = EventLoopBuilder::new().with_android_app(app).build().unwrap();
    pollster::block_on(runner::common_main(event_loop));
}