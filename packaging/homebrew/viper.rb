# Rendered by .github/workflows/homebrew.yml and pushed to ivxlabs/homebrew-tap as Formula/viper.rb.
# __VERSION__, __SHA256_MACOS__ and __SHA256_LINUX__ are filled in there.
class Viper < Formula
  desc "Analysis engine for Difflicit: writes up the changes between two versions as diffmd"
  homepage "https://github.com/ivxlabs/viper"
  version "__VERSION__"
  license "MIT"

  on_macos do
    url "https://github.com/ivxlabs/viper/releases/download/v#{version}/viper-v#{version}-macos-universal.tar.gz"
    sha256 "__SHA256_MACOS__"
  end

  on_linux do
    on_intel do
      url "https://github.com/ivxlabs/viper/releases/download/v#{version}/viper-v#{version}-linux-x86_64.tar.gz"
      sha256 "__SHA256_LINUX__"
    end
  end

  def install
    bin.install "viper"
    doc.install "README.md"
  end

  def caveats
    <<~EOS
      Viper compares files with git, and binaries function by function with radare2:
        brew install radare2
      Difflicit finds viper on its own: pick it as the engine when you Analyze.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/viper --version")
    assert_match "\"protocol\": 1", shell_output("#{bin}/viper --about")
  end
end
