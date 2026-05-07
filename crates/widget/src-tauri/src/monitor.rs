use tauri::LogicalPosition;

pub fn is_position_on_any_monitor(app: &tauri::App, pos: LogicalPosition<f64>) -> bool {
    match app.available_monitors() {
        Ok(monitors) => monitors.iter().any(|m| {
            let scale_factor = m.scale_factor();
            let m_pos = m.position().to_logical(scale_factor);
            let m_size: tauri::LogicalSize<f64> = m.size().to_logical(scale_factor);
            pos.x >= m_pos.x
                && pos.x < m_pos.x + m_size.width
                && pos.y >= m_pos.y
                && pos.y < m_pos.y + m_size.height
        }),
        Err(_) => false,
    }
}
