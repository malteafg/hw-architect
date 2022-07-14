use hw_architect::run;
use pollster;

fn main() {
    pollster::block_on(run());
}
