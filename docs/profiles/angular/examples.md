# `angular@0.1` Examples

## Valid contract

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area movie-reel
  area show-times
  area reserve-tickets
  allow show-times -> movie-reel
  require reserve-tickets -> movie-reel
  exception allow reserve-tickets -> movie-reel {
    through public-api
    reason "Reservations display movie summaries"
  }
}
```

## Invalid contract

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area movie-reel
  allow show-times -> movie-reel
}
```

`show-times` was not declared as an area, so the compiler reports an unresolved
symbol.
