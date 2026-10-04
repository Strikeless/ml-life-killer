# ml-life-killer liblife
A ✨slightly faster✨ simulation library for [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) and [all the copycats](https://en.wikipedia.org/wiki/Life-like_cellular_automaton).

## How does it work?
TODO: Explain the boring stuff

## Is it 🚀✨blazingly fast✨💨 ??
No. If you want to simulate a crazy amount of ticks/generations of CGoL without external state mutations, look into a competent implementation of Hashlife.
If you also need to mutate board state between generations, you still wouldn't lose anything by using a Hashlife implementation, though gains may be smaller.

I haven't done much research on alternative algorithms and this implementation isn't excessively optimized, so you'll probably strike gold somewhere else if you just need speed.

This library aims to be fast enough, not the fastest one there is. It's pretty competent compared to a naive implementation of CGoL, but I'd still place my bets elsewhere.
