use rhdl_doodle::app::App;

fn main() -> eframe::Result {
    let mut app = App::new("".into());
    eframe::run_native(
        "Schematic View",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            if let Some(storage) = cc.storage
                && let Some(scene_rect_str) = storage.get_string("scene_rect")
                && let Ok(scene_rect) = serde_json::from_str::<egui::Rect>(&scene_rect_str)
            {
                app.scene_rect = scene_rect;
            }
            Ok(Box::new(app))
        }),
    )
}
