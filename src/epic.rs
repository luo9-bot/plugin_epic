use ureq;

use serde_json::Value;

/// 游戏信息结构体
#[derive(Debug, Clone)]
pub struct GameInfo {
    /// 游戏名称
    pub title: String,
    /// 游戏描述
    pub description: String,
    /// 开发商/发行商
    pub seller: String,
    /// 免费结束时间
    pub free_end_date: String,
    /// 游戏商店链接
    pub store_url: String,
    /// 预览图片URL
    pub preview_image: String,
}

fn parse_free_games(json_data: &Value) -> Vec<GameInfo> {
    let mut free_games = Vec::new();
    
    let elements = match json_data["data"]["Catalog"]["searchStore"]["elements"].as_array() {
        Some(elements) => elements,
        None => return free_games,
    };
    
    for element in elements {
        if is_free_game(element) {
            if let Some(game_info) = extract_game_info(element) {
                free_games.push(game_info);
            }
        }
    }
    
    free_games
}

fn is_free_game(element: &Value) -> bool {
    // 检查是否有促销优惠
    let has_promotional_offers = element["promotions"]["promotionalOffers"]
        .as_array()
        .map_or(false, |offers| !offers.is_empty());
    
    // 折扣价格是否为0
    let is_free_price = element["price"]["totalPrice"]["discountPrice"]
        .as_u64()
        .map_or(false, |price| price == 0);
    
    has_promotional_offers && is_free_price
}

fn extract_game_info(element: &Value) -> Option<GameInfo> {
    // 游戏名称
    let title = element["title"].as_str()?.to_string();
    
    // 游戏描述
    let description = element["description"].as_str().unwrap_or("").to_string();
    
    // 开发商/发行商
    let seller = element["seller"]["name"].as_str()?.to_string();
    
    // 免费结束时间
    let free_end_date = extract_free_end_date(element)?;
    
    // 生成游戏商店链接
    let store_url = generate_store_url(element);
    
    // 预览图片
    let preview_image = extract_preview_image(element);
    
    Some(GameInfo {
        title,
        description,
        seller,
        free_end_date,
        store_url,
        preview_image,
    })
}

/// 免费结束时间
fn extract_free_end_date(element: &Value) -> Option<String> {
    let promotions = element["promotions"]["promotionalOffers"].as_array()?;
    
    for promotion in promotions {
        let offers = promotion["promotionalOffers"].as_array()?;
        for offer in offers {
            if let Some(end_date) = offer["endDate"].as_str() {
                return Some(end_date.to_string());
            }
        }
    }
    
    None
}

fn generate_store_url(element: &Value) -> String {
    if let Some(product_slug) = element["productSlug"].as_str() {
        return format!("https://store.epicgames.com/zh-CN/p/{}", product_slug);
    }
    
    if let Some(url_slug) = element["urlSlug"].as_str() {
        return format!("https://store.epicgames.com/zh-CN/{}", url_slug);
    }

    if let Some(mappings) = element["offerMappings"].as_array() {
        for mapping in mappings {
            if let Some(page_slug) = mapping["pageSlug"].as_str() {
                return format!("https://store.epicgames.com/zh-CN/p/{}", page_slug);
            }
        }
    }
    
    "https://store.epicgames.com/zh-CN/".to_string()
}


fn extract_preview_image(element: &Value) -> String {
    if let Some(key_images) = element["keyImages"].as_array() {
        for image in key_images {
            if let Some(image_type) = image["type"].as_str() {
                if image_type == "OfferImageWide" || image_type == "OfferImageTall" {
                    if let Some(url) = image["url"].as_str() {
                        return url.to_string();
                    }
                }
            }
        }
        
        if let Some(first_image) = key_images.first() {
            if let Some(url) = first_image["url"].as_str() {
                return url.to_string();
            }
        }
    }
    
    "".to_string()
}

/// 格式化游戏信息为字符串
fn format_game_info(game: &GameInfo) -> String {
    format!(
        "游戏名称: {}\n描述: {}\n开发商/发行商: {}\n免费结束时间: {}\n商店链接: {}\n预览图片: {}",
        game.title,
        game.description,
        game.seller,
        game.free_end_date,
        game.store_url,
        game.preview_image
    )
}

pub fn get_epic_free_games() -> Result<Vec<GameInfo>, Box<dyn std::error::Error>> {
    let url = "https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions";
    
    let response: String = ureq::get(url)
        .query("locale", "zh-CN")
        .query("country", "CN")
        .query("allowCountries", "CN")
        .header("Referer", "https://www.epicgames.com/store/zh-CN/")
        .header("Content-Type", "application/json; charset=utf-8")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .call()?
        .body_mut()
        .read_to_string()?;

    let json: Value = serde_json::from_str(&response)?;
    let free_games = parse_free_games(&json);
    
    Ok(free_games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_free_game() {
        // 测试免费游戏检测
        let free_game = json!({
            "promotions": {
                "promotionalOffers": [{
                    "promotionalOffers": [{
                        "endDate": "2026-04-16T15:00:00.000Z"
                    }]
                }]
            },
            "price": {
                "totalPrice": {
                    "discountPrice": 0
                }
            }
        });
        
        assert!(is_free_game(&free_game));
        
        // 测试非免费游戏（价格不为0）
        let paid_game = json!({
            "promotions": {
                "promotionalOffers": [{
                    "promotionalOffers": [{
                        "endDate": "2026-04-16T15:00:00.000Z"
                    }]
                }]
            },
            "price": {
                "totalPrice": {
                    "discountPrice": 1000
                }
            }
        });
        
        assert!(!is_free_game(&paid_game));
        
        // 测试没有促销的游戏
        let no_promo_game = json!({
            "promotions": {
                "promotionalOffers": []
            },
            "price": {
                "totalPrice": {
                    "discountPrice": 0
                }
            }
        });
        
        assert!(!is_free_game(&no_promo_game));
    }

    #[test]
    fn test_extract_game_info() {
        let game_data = json!({
            "title": "Test Game",
            "description": "This is a test game",
            "seller": {
                "name": "Test Developer"
            },
            "promotions": {
                "promotionalOffers": [{
                    "promotionalOffers": [{
                        "endDate": "2026-04-16T15:00:00.000Z"
                    }]
                }]
            },
            "productSlug": "test-game",
            "keyImages": [{
                "type": "OfferImageWide",
                "url": "https://example.com/image.jpg"
            }]
        });
        
        let game_info = extract_game_info(&game_data).unwrap();
        
        assert_eq!(game_info.title, "Test Game");
        assert_eq!(game_info.description, "This is a test game");
        assert_eq!(game_info.seller, "Test Developer");
        assert_eq!(game_info.free_end_date, "2026-04-16T15:00:00.000Z");
        assert_eq!(game_info.store_url, "https://store.epicgames.com/zh-CN/p/test-game");
        assert_eq!(game_info.preview_image, "https://example.com/image.jpg");
    }

    #[test]
    fn test_format_game_info() {
        let game = GameInfo {
            title: "Test Game".to_string(),
            description: "This is a test game".to_string(),
            seller: "Test Developer".to_string(),
            free_end_date: "2026-04-16T15:00:00.000Z".to_string(),
            store_url: "https://store.epicgames.com/zh-CN/p/test-game".to_string(),
            preview_image: "https://example.com/image.jpg".to_string(),
        };
        
        let formatted = format_game_info(&game);
        assert!(formatted.contains("游戏名称: Test Game"));
        assert!(formatted.contains("描述: This is a test game"));
        assert!(formatted.contains("开发商/发行商: Test Developer"));
        assert!(formatted.contains("免费结束时间: 2026-04-16T15:00:00.000Z"));
        assert!(formatted.contains("商店链接: https://store.epicgames.com/zh-CN/p/test-game"));
        assert!(formatted.contains("预览图片: https://example.com/image.jpg"));
    }

    #[test]
    fn test_get_epic() {
        let free_games = get_epic_free_games().unwrap();
        let mut output = String::new();
        if free_games.is_empty() {
            output.push_str("！！！当前没有免费游戏可领取\n");
        } else {
            output.push_str(&format!("发现 {} 款免费游戏:\n\n", free_games.len()));
            for (i, game) in free_games.iter().enumerate() {
                output.push_str(&format!("【第 {} 款】\n", i + 1));
                output.push_str(&format_game_info(game));
                output.push_str("\n\n");
            }
        }
        println!("{}", output);
    }
}
