pub mod core;
pub mod epic;

use luo9_sdk::Bot;
use luo9_sdk::Msg;
use luo9_sdk::command::Command;
use luo9_sdk::command::PrefixMode;

use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;

pub fn handle_group_msg(group_id: u64, _user_id: u64, msg: &str) {
    match Command::parse(msg, "epic", PrefixMode::None) {
        Some(_cmd) => {
            let free_games = epic::get_epic_free_games().unwrap();
            let mut game_str = String::new();

            let mut msg = CString::new(format!("当前共有{}个免费游戏", free_games.len()).as_str()).unwrap();
            Bot::send_group_msg(group_id, msg);
            sleep(Duration::from_millis(100));
            
            for game in free_games {
                game_str.push_str(&format!("{:?}\n", game));
                msg = Msg::at(_user_id).endl()
                .txt(format!("游戏名称: {}", game.title)).endl()
                .txt(format!("描述: {}", game.description)).endl()
                .txt(format!("开发商/发行商: {}", game.seller)).endl()
                .txt(format!("免费结束时间: {}", game.free_end_date)).endl()
                .txt(format!("商店链接: {}", game.store_url)).endl()
                .txt("预览图片：").image(game.preview_image)
                .build();

                Bot::send_group_msg(group_id, msg);
            }

        }
        None => return,
    };
}