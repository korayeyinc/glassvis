//! UI module for Glassvis.

use std::string::String;

use cairo;
use gdk;
use gio;
use gtk;

use gio::prelude::*;
use gtk::prelude::*;

use gtk::{
    AboutDialog, Adjustment, ApplicationWindow, Box, Builder, Button,
    ColorButton, DrawingArea, EventBox, FileChooserAction,
    FileChooserDialog, FileFilter, Grid, Image, InfoBar,
    Label, MessageType, ResponseType, Scale, Switch,
};

use cairo::{Context};
use gdk_pixbuf::{InterpType, Pixbuf};

mod aux;
mod imgfx;

/// Displays info about defect rate.
fn display_info(info_panel: &InfoBar, defect_info: &Label, width: u32, height: u32, ndiff: u32) {
    let dim = width * height / 100;
    let defect_rate = ndiff as f32 / dim as f32;
    info_panel.set_message_type(MessageType::Error);
    let msg = format!("Total Defect Rate = {}%", defect_rate);
    defect_info.set_text(&msg);
}

/// Displays info about click event position.
fn display_position(info_panel: &InfoBar, pos_info: &Label, xp: i32, yp: i32) {
    info_panel.set_message_type(MessageType::Info);
    let msg = format!("Position: [ X = {:?},  Y = {:?} ]", xp, yp);
    pos_info.set_text(&msg);
}

/// Draws a ruler above image view widget.
fn draw_top_ruler(ruler: &DrawingArea, ctx: &Context) -> Inhibit {
    let width  = ruler.get_allocated_width() as f64;
    let height = ruler.get_allocated_height() as f64;
    let mut x = 0.0;
    
    ctx.set_source_rgb(0.70, 0.80, 0.90);
    ctx.rectangle(x, x, width, height);
    ctx.fill();
    
    let p = height / 2.0;
    let q = height / 3.0;
    let t = height / 4.0;
    
    while x <= width {
        ctx.move_to(x, 0.0);
        
        if x % 40.0 == 0.0 {
            ctx.set_source_rgb(0.10, 0.10, 0.10);
            ctx.set_line_width(1.0);
            ctx.line_to(x, p);
        } else if x % 20.0 == 0.0 {
            ctx.set_source_rgb(0.15, 0.15, 0.15);
            ctx.set_line_width(0.75);
            ctx.line_to(x, q);
        } else {
            ctx.set_source_rgb(0.20, 0.20, 0.20);
            ctx.set_line_width(0.50);
            ctx.line_to(x, t);
        }
        
        ctx.stroke();
        x += 4.0;
    }
    
    Inhibit(false)
}

/// Draws a ruler on the right side of image view widget.
fn draw_right_ruler(ruler: &DrawingArea, ctx: &Context) -> Inhibit {
    let width  = ruler.get_allocated_width() as f64;
    let height = ruler.get_allocated_height() as f64;
    let mut y = 0.0;
    
    ctx.set_source_rgb(0.70, 0.80, 0.90);
    ctx.rectangle(y, y, width, height);
    ctx.fill();
    
    let p = width * 0.5;
    let q = width * 0.66;
    let t = width * 0.75;
    
    while y <= height {
        if y % 40.0 == 0.0 {
            ctx.set_source_rgb(0.10, 0.10, 0.10);
            ctx.set_line_width(1.0);
            ctx.move_to(p, y);
        } else if y % 20.0 == 0.0 {
            ctx.set_source_rgb(0.15, 0.15, 0.15);
            ctx.set_line_width(0.75);
            ctx.move_to(q, y);
        } else {
            ctx.set_source_rgb(0.20, 0.20, 0.20);
            ctx.set_line_width(0.50);
            ctx.move_to(t, y);
        }
        
        ctx.line_to(width, y);
        ctx.stroke();
        y += 4.0;
    }
    
    Inhibit(false)
}

/// Processes input images in rgba color mode and finds their diffs.
fn proc_img(input: &str) -> String {
    let mut src = imgfx::open(&input);
    src = imgfx::resize(src, 600, 800);
    let prefix = "_";
    let path = aux::set_path(&prefix, &input);
    let dst = path.to_str().unwrap();
    imgfx::save(&src, &dst);
    let output = dst.to_string();

    return output;
}

/// Handles about button click event.
fn about_btn_clicked() {
    let dialog = AboutDialog::new();
    dialog.set_program_name("Glassvis");
    dialog.set_version("Version 1.0");
    dialog.set_comments("Computer vision application for visual quality control.");
    dialog.set_website("...");
    dialog.set_license("Copyright © 2019 Koray Eyinç");
    dialog.show();
    dialog.run();
    dialog.hide();
}

/// Handles image button click event.
fn image_btn_clicked(window: &ApplicationWindow, img: &Image) -> String {
    let dialog = FileChooserDialog::new(Some("Open Image"), Some(window), FileChooserAction::Open);

    dialog.add_buttons(&[
        ("Open", ResponseType::Ok.into()),
        ("Cancel", ResponseType::Cancel.into()),
    ]);

    let path = aux::get_path();

    // set file filter with supported image types
    let file_filter = FileFilter::new();
    file_filter.add_pattern("*.bmp");
    file_filter.add_pattern("*.gif");
    file_filter.add_pattern("*.jpg");
    file_filter.add_pattern("*.jpeg");
    file_filter.add_pattern("*.png");
    file_filter.add_pattern("*.pnm");
    file_filter.add_pattern("*.tga");
    file_filter.add_pattern("*.tiff");
    file_filter.add_pattern("*.webp");

    dialog.set_filter(&file_filter);
    dialog.set_current_folder(path);
    dialog.set_select_multiple(false);
    dialog.run();

    let dia = dialog.clone();

    // handle 'cancel' button click event
    dialog.connect_response(move |_, resp_type| {
        if resp_type == ResponseType::Cancel.into() {
            dia.destroy();
            return;
        }
    });

    let file_name = dialog.get_filename().unwrap();
    let input = file_name.to_str().unwrap();
    let mut output = String::from("");

    // check if input is an image file
    if aux::is_image_file(input) {
        output = proc_img(input);
        img.set_property_file(Some(output.as_str()));
    }

    dialog.destroy();
    return output;
}

/// Handles diff button click event.
fn diff_btn_clicked(
    info_panel: &InfoBar,
    defect_info: &Label,
    ref_img: &Image,
    capt_img: &Image,
    significance: u8,
    bound_state: bool,
) {
    let ref_img_file = ref_img.get_property_file().unwrap();
    let ref_img_path = ref_img_file.as_str().to_string();

    let capt_img_file = capt_img.get_property_file().unwrap();
    let mut capt_img_path = capt_img_file.as_str().to_string();

    // normalize captured image path if necessary
    if capt_img_path.contains("diff") {
        capt_img_path = ref_img_path.replace("_ref", "_capt");
    }

    let mut src = imgfx::open(&ref_img_path);
    let mut dst = imgfx::open(&capt_img_path);

    // mark diff pixels with specified color
    let (diff, points, width, height, counter) = imgfx::mark_diffs(&mut src, &mut dst, significance);

    let prefix = "diff";
    let path = aux::set_path(&prefix, &capt_img_path);
    let diff_path = path.to_str().unwrap().to_string();
    
    let key = String::from("diff_path");
    let val = &diff_path;
    aux::set_env(&key, val);
    
    let active = String::from("active");
    aux::set_env(&active, val);

    if bound_state == true {
        let rect  = imgfx::get_box(points);
        dst = imgfx::draw_rect(diff, rect);
        imgfx::save(&dst, &diff_path);
    } else {
        imgfx::save(diff, &diff_path);
    }
    
    capt_img.set_from_file(aux::to_path(&diff_path));
    
    // calculate defect rate and display it inside info panel
    display_info(info_panel, defect_info, width, height, counter);
}

/// Handles config button click event.
fn config_btn_clicked(settings_panel: &Grid) {
    if settings_panel.get_visible() == false {
        settings_panel.show();
    } else {
        settings_panel.hide();
    }
}

/// Handles zoom-in button click event.
fn zoom_in_btn_clicked(ref_img: &Image, capt_img: &Image, in_width: i32, in_height: i32) {
    let mut ref_pixbuf: Pixbuf = ref_img.get_pixbuf().unwrap();
    let ref_width = ref_pixbuf.get_width() + in_width;
    let ref_height = ref_pixbuf.get_height() + in_height;

    let mut capt_pixbuf: Pixbuf = capt_img.get_pixbuf().unwrap();
    let capt_width = capt_pixbuf.get_width() + in_width;
    let capt_height = capt_pixbuf.get_height() + in_height;
    
    // refresh pixbuf before zooming
    zoom_fit_btn_clicked(ref_img, capt_img);
    ref_pixbuf = ref_img.get_pixbuf().unwrap();
    capt_pixbuf = capt_img.get_pixbuf().unwrap();
    
    ref_pixbuf = ref_pixbuf
        .scale_simple(ref_width, ref_height, InterpType::Hyper)
        .unwrap();
    
    capt_pixbuf = capt_pixbuf
        .scale_simple(capt_width, capt_height, InterpType::Hyper)
        .unwrap();
    
    ref_img.set_from_pixbuf(Some(&ref_pixbuf));
    capt_img.set_from_pixbuf(Some(&capt_pixbuf));
}

/// Handles zoom-out button click event.
fn zoom_out_btn_clicked(ref_img: &Image, capt_img: &Image, out_width: i32, out_height: i32) {
    let mut ref_pixbuf: Pixbuf = ref_img.get_pixbuf().unwrap();
    let ref_width = ref_pixbuf.get_width() - out_width;
    let ref_height = ref_pixbuf.get_height() - out_height;
    
    let mut capt_pixbuf: Pixbuf = capt_img.get_pixbuf().unwrap();
    let capt_width = capt_pixbuf.get_width() - out_width;
    let capt_height = capt_pixbuf.get_height() - out_height;
    
    // refresh pixbuf before zooming
    zoom_fit_btn_clicked(ref_img, capt_img);
    ref_pixbuf = ref_img.get_pixbuf().unwrap();
    capt_pixbuf = capt_img.get_pixbuf().unwrap();
    
    ref_pixbuf = ref_pixbuf
        .scale_simple(ref_width, ref_height, InterpType::Hyper)
        .unwrap();
    
    capt_pixbuf = capt_pixbuf
        .scale_simple(capt_width, capt_height, InterpType::Hyper)
        .unwrap();

    ref_img.set_from_pixbuf(Some(&ref_pixbuf));
    capt_img.set_from_pixbuf(Some(&capt_pixbuf));
}

/// Handles zoom-fit button click event.
fn zoom_fit_btn_clicked(ref_img: &Image, capt_img: &Image) {
    let mut key = String::from("ref_path");
    let ref_val = aux::get_env(&key);
    let ref_path = aux::to_path(&ref_val);
    
    key = String::from("active");
    let capt_val = aux::get_env(&key);
    let capt_path = aux::to_path(&capt_val);

    ref_img.set_from_file(ref_path);
    capt_img.set_from_file(capt_path);
}

/// Handles fullscreen button click event.
fn fullscreen_btn_clicked(fullscreen_btn: &Button, _window: &ApplicationWindow) {
    let fullscreen = String::from("fullscreen");
    let on  = String::from("on");
    let off = String::from("off");
    
    if aux::get_env(&fullscreen) == off {
        aux::set_env(&fullscreen, &on);
        _window.fullscreen();
    } else {
        aux::set_env(&fullscreen, &off);
        _window.unfullscreen();
    }
}

/// Handles app window -> quit button click event.
fn quit(_window: &ApplicationWindow, _event: &gdk::Event) -> Inhibit {
    _window.destroy();
    Inhibit(false)
}

/// Builds GTK UI from Glade design.
pub fn build(application: &gtk::Application) {
    let glade_src = include_str!("ui/ui.glade");
    let builder = Builder::new_from_string(glade_src);

    // Application window
    let window: ApplicationWindow = builder
        .get_object("AppWin")
        .expect("Couldn't get application window!");

    // Container box
    let root_box: Box = builder
        .get_object("RootBox")
        .expect("Couldn't get rootbox widget!");

    //--------------------------------------------------------------------------
    // Panel widgets

    // Settings panel widget
    let settings_panel: Grid = builder
        .get_object("SettingsPanel")
        .expect("Couldn't get settings panel widget!");

    // Headerbar widgets
    let image_btn: Button = builder
        .get_object("ImageBtn")
        .expect("Couldn't get image button widget!");

    let camera_btn: Button = builder
        .get_object("CameraBtn")
        .expect("Couldn't get camera button widget!");

    let diff_btn: Button = builder
        .get_object("DiffBtn")
        .expect("Couldn't get diff button widget!");

    let zoom_in_btn: Button = builder
        .get_object("ZoomInBtn")
        .expect("Couldn't get zoom-in button widget!");

    let zoom_out_btn: Button = builder
        .get_object("ZoomOutBtn")
        .expect("Couldn't get zoom-out button widget!");

    let zoom_fit_btn: Button = builder
        .get_object("ZoomFitBtn")
        .expect("Couldn't get zoom-fit button widget!");

    let config_btn: Button = builder
        .get_object("ConfigBtn")
        .expect("Couldn't get config button widget!");

    let about_btn: Button = builder
        .get_object("AboutBtn")
        .expect("Couldn't get about button widget!");
    
    let fullscreen_btn: Button = builder
        .get_object("FullscreenBtn")
        .expect("Couldn't get fullscreen button widget!");
    
    let key = String::from("fullscreen");
    let val = String::from("off");
    aux::set_env(&key, &val);
    
    let quit_btn: Button = builder
        .get_object("QuitBtn")
        .expect("Couldn't get quit button widget!");

    // Settings panel widgets
    let slider: Scale = builder
        .get_object("Slider")
        .expect("Couldn't get significance slider widget!");

    let adjuster: Adjustment = builder
        .get_object("Adjuster")
        .expect("Couldn't get adjustment widget!");

    // set optimum defect significance
    let value: f64 = 10.0;
    adjuster.set_value(value);
    
    let bound_box: Switch = builder
        .get_object("BoundingBox")
        .expect("Couldn't get switch widget!");

    let color_btn: ColorButton = builder
        .get_object("MarkerColor")
        .expect("Couldn't get color picker widget!");

    // Image view widgets
    let top_ruler: DrawingArea = builder
        .get_object("TopRuler")
        .expect("Couldn't get ruler widget!");
    
    let right_ruler: DrawingArea = builder
        .get_object("RightRuler")
        .expect("Couldn't get ruler widget!");
    
    let ref_evt: EventBox = builder
        .get_object("RefEventBox")
        .expect("Couldn't get reference event box widget!");
    
    let capt_evt: EventBox = builder
        .get_object("CaptEventBox")
        .expect("Couldn't get captured event box widget!");
    
    let ref_img: Image = builder
        .get_object("RefImage")
        .expect("Couldn't get reference image widget!");

    let capt_img: Image = builder
        .get_object("CaptImage")
        .expect("Couldn't get captured image widget!");

    // Info panel widget
    let info_panel: InfoBar = builder
        .get_object("InfoPanel")
        .expect("Couldn't get info panel widget!");

    let defect_info: Label = builder
        .get_object("DefectInfo")
        .expect("Couldn't get defect info label widget!");
    
    let position_info: Label = builder
        .get_object("PositionInfo")
        .expect("Couldn't get position info label widget!");

    //--------------------------------------------------------------------------
    // event handlers

    window.set_default_size(1280, 800);
    window.maximize();
    window.set_application(application);
    window.connect_delete_event(quit);
    
    let win_img = window.clone();
    let win_cam = window.clone();
    let win_full = window.clone();
    let win_quit = window.clone();

    let mut ref_img1 = Image::new();
    ref_img1.clone_from(&ref_img);

    let mut ref_img2 = Image::new();
    ref_img2.clone_from(&ref_img);

    let mut ref_img3 = Image::new();
    ref_img3.clone_from(&ref_img);

    let mut ref_img4 = Image::new();
    ref_img4.clone_from(&ref_img);

    let mut ref_img5 = Image::new();
    ref_img5.clone_from(&ref_img);

    let mut capt_img1 = Image::new();
    capt_img1.clone_from(&capt_img);

    let mut capt_img2 = Image::new();
    capt_img2.clone_from(&capt_img);

    let mut capt_img3 = Image::new();
    capt_img3.clone_from(&capt_img);

    let mut capt_img4 = Image::new();
    capt_img4.clone_from(&capt_img);

    let mut capt_img5 = Image::new();
    capt_img5.clone_from(&capt_img);
    
    let info_panel1 = info_panel.clone();
    let info_panel2 = info_panel.clone();
    let pos_info = position_info.clone();
    
    top_ruler.connect_draw(draw_top_ruler);
    right_ruler.connect_draw(draw_right_ruler);
    
    ref_evt.connect_button_press_event(move |_, evt_btn| {
        let (xp, yp) = evt_btn.get_position();
        display_position(&info_panel1, &pos_info, xp as i32, yp as i32);
        Inhibit(false)
    });
    
    capt_evt.connect_button_press_event(move |_, evt_btn| {
        let (xp, yp) = evt_btn.get_position();
        display_position(&info_panel2, &position_info, xp as i32, yp as i32);
        Inhibit(false)
    });
    
    image_btn.connect_clicked(move |_| {
        let ref_path = image_btn_clicked(&win_img, &ref_img1);
        let key = String::from("ref_path");
        let val = ref_path;
        aux::set_env(&key, &val);
    });

    camera_btn.connect_clicked(move |_| {
        let capt_path = image_btn_clicked(&win_cam, &capt_img1);
        let key = String::from("capt_path");
        let val = capt_path;
        aux::set_env(&key, &val);
        let active = String::from("active");
        aux::set_env(&active, &val);
    });

    diff_btn.connect_clicked(move |_| {
        let significance = adjuster.get_value() as u8;
        let bound_state = bound_box.get_state();
        
        zoom_fit_btn_clicked(&ref_img2, &capt_img2);
        
        diff_btn_clicked(
            &info_panel,
            &defect_info,
            &ref_img2,
            &capt_img2,
            significance,
            bound_state,
        );
    });
    
    let zoom_width: i32 = 60;
    let zoom_height: i32 = 80;
    
    zoom_in_btn.connect_clicked(move |_| zoom_in_btn_clicked(&ref_img3, &capt_img3, zoom_width, zoom_height));

    zoom_out_btn.connect_clicked(move |_| zoom_out_btn_clicked(&ref_img4, &capt_img4, zoom_width, zoom_height));

    zoom_fit_btn.connect_clicked(move |_| zoom_fit_btn_clicked(&ref_img5, &capt_img5));

    config_btn.connect_clicked(move |_| config_btn_clicked(&settings_panel));

    about_btn.connect_clicked(move |_| about_btn_clicked());
    
    fullscreen_btn.connect_clicked(move |btn| fullscreen_btn_clicked(&btn, &win_full));
    
    quit_btn.connect_clicked(move |_| {
        win_quit.destroy();
        Inhibit(false);
    });
    
    window.show_all();
}
