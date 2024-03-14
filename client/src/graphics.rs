/*
** Animated sprites data structure :
** Goal is to associate the AnimatedSprites map layer from the exported JSON (from aseprite)
** to their sprites data (frames, duration) and an Enum.je 
*/

// pub enum AnimatedSprites {
//     RedFlowerBgDark,
//     BlueFlowerBgDark,
//     RedFlower,
//     BlueFlower,    
// }

// pub struct AnimatedSpritesAsset {
//     filename: String,
// }

// pub struct SpritesAssetsLookup {
//     list: Vec<AnimatedSpritesAsset>
// }

// impl SpritesAssetsLookup {

//     pub fn new(&mut self) -> SpritesAssetsLookup {
//         return SpritesAssetsLookup {
//             list: self.make_list()
//         }
//     }

//     fn make_list(&mut self) -> Vec<AnimatedSpritesAsset> {
//         let mut list: Vec<AnimatedSpritesAsset> = Vec::with_capacity(4);
//         list[0] = AnimatedSpritesAsset { filename: String::from("RedFlowerBgDark.json") }; // AnimatedSprites::RedFlowerBgDark
//         list[1] = AnimatedSpritesAsset { filename: String::from("BlueFlowerBgDark.json") }; // AnimatedSprites::BlueFlowerBgDark
//         list[2] = AnimatedSpritesAsset { filename: String::from("RedFlower.json") }; // AnimatedSprites::RedFlower
//         list[3] = AnimatedSpritesAsset { filename: String::from("BlueFlower.json") }; // AnimatedSprites::BlueFlower

//         return list;
//     }
// }