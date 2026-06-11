extern crate clap;
mod torrent;
mod tracker;
mod utils;
mod peer;
// mod storage;

use crate::torrent::{parse_torrent};
use crate::tracker::udp::udp_announce_from_torrent;
use crate::utils::sha1::sha1_batch;
use crate::peer::connection::PeerConnection;

use clap::Parser;
use tokio;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Parse a .torrent file and print its contents")]
struct Args {
    /// Path to the .torrent file
    torrent: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Read the torrent file.
    // Parse its contents.
    // Use UDP if tracker URL starts with "udp://".
    // Get peer list from tracker.
    // Get and Store the individual pieces sha1 hashes.

    let data = match std::fs::read(&args.torrent) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read {}: {}", args.torrent.display(), e);
            std::process::exit(1);
        }
    };

    // Call the parser and print the result to stdout.
    // This will work whether parse_torrent returns a Torrent or a Result<Torrent, E>
    // as long as the returned type implements Debug.
    let torrent = match parse_torrent(data.as_slice()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to parse torrent: {:?}", e);
            std::process::exit(1);
        }
    };

    // Use UDP tracker to get peers
    let peers = match udp_announce_from_torrent(&torrent, 6881) {
        Ok(peers) => {
            peers
        }
        Err(e) => {
            eprintln!("Failed to get peers from UDP tracker: {}", e);
            std::process::exit(1);
        }
    };

    // Sha1 pieces fingerprints
    let pieces = match &torrent.info.pieces {
        Some(p) => p.as_slice(),
        None => {
            eprintln!("Torrent file has no 'pieces' data");
            std::process::exit(1);
        }
    };

    // Split the concatenated pieces blob into 20-byte chunks (SHA1 hashes) for V1 BitTorrent
    let piece_slices: Vec<&[u8]> = pieces.chunks(20).collect();
    let piece_sha1s = sha1_batch(&piece_slices);

    // println!("Piece SHA1 hashes: {:#?}", torrent.info);

    // 1. Fetch a piece.
    // 2. Verify the piece is valid.
    // 3. Store the piece.
    // 4. Take a new piece and Repeat.

    // 1. Fetch a piece
    for peer in &peers {
        println!("Peer: {}", peer);
        let peer_connection: PeerConnection = match PeerConnection::connect(*peer, piece_sha1s[0], [0u8; 20]).await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to connect to peer {}: {}", *peer, e);
                continue;
            }
        };
    }
    

}
