# Test file with various performance issues

let
  # Import from derivation (IFD)
  generatedConfig = import (pkgs.runCommand "config" {} ''
    echo '{ foo = "bar"; }' > $out
  '');

  # Excessive string concatenation
  longString = "a" + "b" + "c" + "d" + "e" + "f" + "g" + "h" + "i" + "j" + 
               "k" + "l" + "m" + "n" + "o" + "p" + "q" + "r" + "s" + "t";

  # Inefficient list operations
  badList = [ 1 2 3 ] ++ [ 4 ] ++ [ 5 ] ++ [ 6 ] ++ [ 7 ] ++ [ 8 ];

  # Nested map operations
  nestedMaps = map (x: map (y: x + y) [ 1 2 3 ]) [ 4 5 6 ];

  # Deep attribute access
  deepValue = config.services.nginx.virtualHosts.example.locations.root.extraConfig.settings.advanced.value;

  # Multiple imports of same file
  import1 = import ./common.nix;
  import2 = import ./common.nix;
  import3 = import ./common.nix;
  import4 = import ./common.nix;

  # Let inside frequently called function
  inefficientFunc = x: 
    let
      # This gets recalculated every call
      expensiveComputation = builtins.foldl' (a: b: a + b) 0 (builtins.genList (i: i) 1000);
    in x + expensiveComputation;

in {
  inherit generatedConfig longString badList nestedMaps deepValue inefficientFunc;
} 