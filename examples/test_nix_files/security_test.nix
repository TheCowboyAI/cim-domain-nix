# Test file with various security issues

{
  # Insecure fetcher without hash
  badFetch1 = builtins.fetchurl {
    url = "https://example.com/source.tar.gz";
  };

  # Weak hash (MD5)
  badFetch2 = builtins.fetchurl {
    url = "https://example.com/source.tar.gz"; 
    md5 = "d41d8cd98f00b204e9800998ecf8427e";
  };

  # SHA1 (deprecated)
  badFetch3 = builtins.fetchurl {
    url = "https://example.com/source.tar.gz";
    sha1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
  };

  # Insecure HTTP URL
  badUrl = builtins.fetchurl {
    url = "http://example.com/source.tar.gz";
    sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
  };

  # Git protocol (unencrypted)
  badGit = builtins.fetchGit {
    url = "git://github.com/example/repo.git";
    sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
  };

  # Impure functions
  currentTime = builtins.currentTime;
  envVar = builtins.getEnv "HOME";
  
  # Good example for comparison
  goodFetch = builtins.fetchurl {
    url = "https://example.com/source.tar.gz";
    sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
  };
}
