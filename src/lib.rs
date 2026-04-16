pub mod core;
pub mod epic;

use luo9_sdk::Bot;
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
                msg = CString::new(format!(
                    "游戏名称: {}\n描述: {}\n开发商/发行商: {}\n免费结束时间: {}\n商店链接: {}\n预览图片: [CQ:image,url={}]", 
                    game.title, 
                    game.description, 
                    game.seller, 
                    game.free_end_date, 
                    game.store_url, 
                    game.preview_image).as_str()).unwrap();

                    Bot::send_group_msg(group_id, msg);
            }

        }
        None => return,
    };
}
