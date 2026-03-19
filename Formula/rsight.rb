class Rsight < Formula
  desc "Fast TUI search for files, content, and AI conversations"
  homepage "https://github.com/olucasandrade/rsight"
  version "0.2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/olucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "dca66c1e1ae159ac439278eaba37fce25808dfc47afdd11ad05465decb9e36d7" # arm64
    else
      url "https://github.com/olucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "700ed61acb662c7ce3a0c3fe8b8d2cfa4ece8088d619408d1346ba4565805057" # x86_64
    end
  end

  def install
    bin.install "rsight"
  end

  test do
    assert_predicate bin/"rsight", :exist?
  end
end
