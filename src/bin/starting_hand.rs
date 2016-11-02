extern crate furry_fiesta;

use furry_fiesta::holdem::StartingHand;

fn main() {
    let hands = StartingHand::all();
    let num_uniq_hands = hands.len();
    let num_hands: usize = hands.iter().map(|h| h.possible_hands().len()).sum();
    for sh in hands {
        println!("{:?}", sh);
        for h in sh.possible_hands() {
            println!("\t {:?}", h);
        }
    }
    println!("StartingHand::all().len() = {:?}", num_uniq_hands);
    println!("Number of hands = {:?}", num_hands);
}
