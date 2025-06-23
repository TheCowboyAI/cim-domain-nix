# Test file with dead code examples

let
  # Used variable
  used = 42;
  
  # Unused variable
  unused = 99;
  
  # Another unused
  neverUsed = "hello";
  
  # Function with unused parameter
  func = x: y: x + 1;  # y is never used
  
  # Unreachable code after throw
  errorFunc = x:
    if x < 0
    then throw "negative number"
    else x;
  
  unreachableValue = 123;  # This line is after throw in some paths

  # Redundant definitions
  duplicate = 1;
  duplicate = 2;  # Redefining the same name
  
in {
  result = used + func 5 10;
  error = errorFunc (-1);
} 