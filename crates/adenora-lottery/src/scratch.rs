use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Instant scratch card game configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchConfig {
    pub name: String,
    pub ticket_price: Decimal,
    pub grid_size: (usize, usize), // e.g., (3, 3) for 3x3 grid
    pub symbols: Vec<String>,
    pub prize_table: Vec<ScratchPrize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchPrize {
    pub matches_needed: usize,
    pub symbol: String,
    pub prize: Decimal,
    pub odds: f64, // probability 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchCard {
    pub grid: Vec<Vec<String>>,
    pub revealed: Vec<Vec<bool>>,
    pub prize: Option<Decimal>,
}

/// Generate a scratch card with predetermined outcome.
/// The prize is determined first, then the grid is populated to match.
pub fn generate_card(config: &ScratchConfig) -> ScratchCard {
    let mut rng = rand::thread_rng();
    let (rows, cols) = config.grid_size;

    // Determine if this card wins (and what prize)
    let prize = determine_prize(config, &mut rng);

    // Generate grid
    let mut grid = vec![vec![String::new(); cols]; rows];

    if let Some((ref winning_symbol, matches_needed)) = prize {
        // Place winning symbols
        let total_cells = rows * cols;
        let mut positions: Vec<usize> = (0..total_cells).collect();
        use rand::seq::SliceRandom;
        positions.shuffle(&mut rng);

        for (i, &pos) in positions.iter().enumerate() {
            let r = pos / cols;
            let c = pos % cols;
            if i < matches_needed {
                grid[r][c] = winning_symbol.clone();
            } else {
                // Random non-winning symbol
                let sym = &config.symbols[rng.gen_range(0..config.symbols.len())];
                grid[r][c] = sym.clone();
            }
        }
    } else {
        // No win — random symbols, ensuring no prize pattern
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                *cell = config.symbols[rng.gen_range(0..config.symbols.len())].clone();
            }
        }
    }

    let prize_amount = prize.as_ref().and_then(|(symbol, _)| {
        config
            .prize_table
            .iter()
            .find(|p| &p.symbol == symbol)
            .map(|p| p.prize)
    });

    ScratchCard {
        grid,
        revealed: vec![vec![false; cols]; rows],
        prize: prize_amount,
    }
}

fn determine_prize(
    config: &ScratchConfig,
    rng: &mut impl Rng,
) -> Option<(String, usize)> {
    let roll: f64 = rng.gen();
    let mut cumulative = 0.0;

    for prize in &config.prize_table {
        cumulative += prize.odds;
        if roll < cumulative {
            return Some((prize.symbol.clone(), prize.matches_needed));
        }
    }

    None // No prize
}
