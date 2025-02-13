class Skeletor < Formula
  desc "A super optimised Rust scaffolding tool with snapshot annotations"
  homepage "https://github.com/jasonnathan/skeletor"
  url "https://github.com/jasonnathan/skeletor/releases/download/v2.0.1/skeletor.tar.gz" # link to your tar.gz binary
  sha256 "169bb439a7e69d08967f891cbbaf2e6537ceec9f9eb736ebbb503c9bca83366c"  # Replace this with the actual SHA256 checksum of your tar.gz file
  license "MIT"

  def install
    bin.install "skeletor"
  end

  test do
    system "#{bin}/skeletor", "--version"
  end
end
