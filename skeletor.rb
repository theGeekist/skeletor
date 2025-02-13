class Skeletor < Formula
  desc "A super optimised Rust scaffolding tool with snapshot annotations"
  homepage "https://github.com/jasonnathan/skeletor"
  url "https://github.com/jasonnathan/skeletor/releases/download/v2.2.8/skeletor-macos-latest-x86_64-apple-darwin.tar.gz" # link to your tar.gz binary.gz" # link to your tar.gz binary
  sha256 "NEW_SHA256_CHECKSUM_HERE"  # Replace with the actual SHA256 checksum of your tar.gz file
  license "MIT"

  def install
    bin.install "skeletor"
  end

  test do
    system "#{bin}/skeletor", "--version"
  end
end
