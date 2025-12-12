use chrono::Local;
use std::io::Write;

pub fn prepare() {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
		.format(log_format)
		.init();
}

fn log_format(buf: &mut env_logger::fmt::Formatter, record: &log::Record) -> std::io::Result<()> {
	let level_style = buf.default_level_style(record.level());
	let level_end = level_style.render_reset();
	let level_start = level_style.render();

	let dark_style = env_logger::fmt::style::Style::new().dimmed();
	let dark_end = dark_style.render_reset();
	let dark_start = dark_style.render();

	writeln!(buf,
		"{dark_start}{}{dark_end} {level_start}{}{level_end} {dark_start}{}{dark_end} {}",
		Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
		record.level(),
		record.module_path().unwrap_or("<unnamed>"),
		record.args()
	)
}
