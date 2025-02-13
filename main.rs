use std::{fs, io, thread, time};
use std::path::Path;
use std::io::BufRead;
use sysinfo::{Process, ProcessExt, System, SystemExt};
use users::get_user_by_uid;
use sysinfo::AsU32;
use std::collections::HashSet;
use std::collections::HashMap;
use eframe::egui;
use egui::RichText;

fn total_cpu_usage(system: &System) -> f32 {
    let mut total_cpu_usage = 0.0;
    let mut seen_pid = HashSet::new();
    for process in system.processes().values() {
        let pid = process.pid();
        if !seen_pid.contains(&pid) {
            total_cpu_usage += process.cpu_usage();
            seen_pid.insert(pid);
        }
    }
    
    total_cpu_usage
}

fn get_user_name_by_pid(pid: i32) -> Option<String> {
    let path = format!("/proc/{}/status", pid);
    let path = Path::new(&path);

    if path.exists() {
        let file = fs::File::open(path).ok()?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.ok()?;
            if line.starts_with("Uid:") {
                let uid_str = line.split_whitespace().nth(1)?;
                let uid: u32 = uid_str.parse().ok()?;

                if let Some(user) = get_user_by_uid(uid) {
                    return Some(user.name().to_string_lossy().to_string()); 
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    exe_path: String,
    user_name: String,
}

impl ProcessInfo {
    fn from_process(process: &Process) -> Self {
        let name = process.name().to_string();
        let exe_path = process.exe().display().to_string();
        let user_name = get_user_name_by_pid(process.pid())
            .unwrap_or_else(|| "Necunoscut".to_string());

        ProcessInfo {
            name,
            exe_path,
            user_name,
        }
    }
}

fn lista(system: &mut System, output: &mut Vec<String>, process_map: &mut HashMap<u32, ProcessInfo>) {

    let mut seen_pids = HashSet::new();

    let total_cpu_used = total_cpu_usage(system);
    let total_memory_used = system.used_memory();
    output.push(format!("  Total CPU used: {:.2}%", total_cpu_used));
    output.push(format!("  Total memory used: {} KB", total_memory_used));
    output.push(format!("  {:<40} {:<10} {:<15} {:<105} {:<17}", "Nume Proces", "CPU (%)", "Memorie (KB)", "Calea Executabilului", "Utilizator"));

    for process in system.processes().values() {
        let pid = process.pid().as_u32();
        if !seen_pids.contains(&pid) {
            let process_info = process_map
            .entry(pid)
            .or_insert_with(|| ProcessInfo::from_process(process));
            let cpu_usage = process.cpu_usage(); 
            let memory = process.memory();

            output.push(format!("  {:<40} {:<10.2} {:<15} {:<105} {:<17}", process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name));
            seen_pids.insert(pid);
        }
    }
}

fn arbore(system: &mut System, output: &mut Vec<String>, process_map: &mut HashMap<u32, ProcessInfo>) {

    let total_cpu_used = total_cpu_usage(system);
    let total_memory_used = system.used_memory();
    output.push(format!("  Total CPU used: {:.2}%", total_cpu_used));
    output.push(format!("  Total memory used: {} KB", total_memory_used));


    let mut process_tree: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut process_names: HashMap<u32, &Process> = HashMap::new();

    for process in system.processes().values() {
        let pid = process.pid().as_u32();
        let ppid_u32 = process.parent().map(|ppid| ppid.as_u32()).unwrap_or(0); 
        
        process_tree.entry(ppid_u32).or_default().push(pid);

        process_names.insert(pid, process);
    }
    
    output.push(format!("  {:<46} {:<10} {:<15} {:<105} {:<17}", "Nume Proces", "CPU (%)", "Memorie (KB)", "Calea Executabilului", "Utilizator"));
    print_tree(&process_tree, &process_names, 0, 0, output, process_map); 
}

fn print_tree(
    tree: &HashMap<u32, Vec<u32>>,
    names: &HashMap<u32, &Process>,
    current_pid: u32,
    depth: usize,
    output: &mut Vec<String>,
    process_map: &mut HashMap<u32, ProcessInfo>
) {
    let indent = "  ".repeat(depth);

    if let Some(process) = names.get(&current_pid) {
        let process_info = process_map
        .entry(current_pid)
        .or_insert_with(|| ProcessInfo::from_process(process));
        let cpu_usage = process.cpu_usage(); 
        let memory = process.memory();

        if depth == 0 { output.push(format!("  {}- {:<44} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 1 { output.push(format!("  {}- {:<42} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 2 { output.push(format!("  {}- {:<40} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 3 { output.push(format!("  {}- {:<38} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 4 { output.push(format!("  {}- {:<36} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 5 { output.push(format!("  {}- {:<34} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
        if depth == 6 { output.push(format!("  {}- {:<32} {:<10.2} {:<15} {:<105} {:<17}", indent, process_info.name, cpu_usage, memory, process_info.exe_path, process_info.user_name)); }    
    } else if current_pid == 0 {
        output.push(format!("  {}- [Sistem] (PID: 0)", indent));
    }

    if let Some(children) = tree.get(&current_pid) {
        for &child_pid in children {
            print_tree(tree, names, child_pid, depth + 1, output, process_map);
        }
    }
}   

struct App {
    show_list: bool,
    show_tree: bool,
    output: Vec<String>,
    update: std::time::Instant,
    system: System, 
    process_map: HashMap<u32, ProcessInfo>
}

impl Default for App {
    fn default() -> Self {
        Self {
            show_list: false,
            show_tree: false,
            output: Vec::new(),
            update: std::time::Instant::now(),
            system: System::new_all(),
            process_map: HashMap::new()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let available_rect = ctx.screen_rect();
        let title_size = available_rect.width() * 0.05;

        ctx.set_style(egui::Style {
            visuals: egui::Visuals {
                override_text_color: Some(egui::Color32::from_rgb(255, 255, 255)),
                panel_fill: egui::Color32::from_rgb(46, 17, 36), 
                ..Default::default()
            },
            ..Default::default()
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            if self.show_list || self.show_tree {
                let now = std::time::Instant::now();
                if now.duration_since(self.update).as_secs() >= 5 {
                    self.output.clear();
                    if self.show_list {
                        lista(&mut self.system,&mut self.output, &mut self.process_map);
                    } else if self.show_tree {
                        arbore(&mut self.system, &mut self.output, &mut self.process_map);
                    }
                    self.system.refresh_all(); 
                    self.update = now;
                }
                let available_rect = ui.available_rect_before_wrap();
                let button_width = available_rect.width() * 0.06; 
                let button_height = available_rect.height() * 0.032; 
            
                let button_1_x = available_rect.left() + 10.0; 
                let button_1_y = available_rect.top() + 10.0; 
                let font_size = button_height * 0.6; 

                let button_1_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(button_1_x, button_1_y),
                    egui::Vec2::new(button_width, button_height),
                );
            

                ui.allocate_ui_at_rect(button_1_rect, |ui| {
                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(RichText::new("Back").size(font_size).strong(),)
                        )
                        .clicked()
                    {
                        self.show_list = false;
                        self.show_tree = false;
                    }
                });
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in &self.output {
                        ui.horizontal(|ui|{
                        ui.label(RichText::new(line).size(available_rect.width() * 0.00855).family(egui::FontFamily::Monospace));});
                    }
                });
            }
            else {
            ui.vertical_centered(|ui| {
                ui.add_space(title_size);
                ui.label(RichText::new("Welcome to TASK MANAGER!").size(title_size));
                ui.add_space(title_size);
                ui.label(RichText::new("How do you want to view the processes?").size(title_size * 0.5));

                let available_rect = ui.available_rect_before_wrap();
                let button_width = available_rect.width() * 0.15; 
                let button_height = available_rect.height() * 0.09; 
            
                let button_1_x = available_rect.left() + available_rect.width() * 0.34; 
                let button_1_y = available_rect.top() + 200.0; 
                let button_2_x = button_1_x + button_width + available_rect.width() * 0.02; 
                let button_2_y = button_1_y; 
                let font_size = button_height * 0.5; 

                let button_1_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(button_1_x, button_1_y),
                    egui::Vec2::new(button_width, button_height),
                );
            
                let button_2_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(button_2_x, button_2_y),
                    egui::Vec2::new(button_width, button_height),
                );

                ui.allocate_ui_at_rect(button_1_rect, |ui| {
                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(RichText::new("List").size(font_size).strong(),)
                        )
                        .clicked()
                    {
                        self.show_list = true;
                        self.show_tree = false;
                        self.output.clear();
                        thread::sleep(time::Duration::from_secs(2)); 
                        self.system.refresh_all(); 
                        lista(&mut self.system, &mut self.output, &mut self.process_map);
                        self.update = std::time::Instant::now();
                    }
                });
            
                ui.allocate_ui_at_rect(button_2_rect, |ui| {
                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new(RichText::new("Tree").size(font_size).strong(),)
                        )
                        .clicked()
                    {
                        self.show_tree = true;
                        self.show_list = false;
                        self.output.clear();
                        thread::sleep(time::Duration::from_secs(2)); 
                        self.system.refresh_all(); 
                        arbore(&mut self.system, &mut self.output, &mut self.process_map);
                        self.update = std::time::Instant::now();
                    }
                });
            }); }
        });
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)), 
        ..Default::default() 
    };

    eframe::run_native(
        "Task Manager",  
        options,  
        Box::new(|_cc| Box::new(App::default())), 
    )
}
