#!/usr/bin/env node

const fs = require('fs')
const path = require('path')
const https = require('https')
const { execSync } = require('child_process')

const package = require('../package.json')

// Determine platform and architecture
function getPlatform() {
  const platform = process.platform
  const arch = process.arch

  if (platform === 'darwin') {
    if (arch === 'x64') {
      return 'macos-x64'
    } else if (arch === 'arm64') {
      return 'macos-arm64'
    }
  } else if (platform === 'linux' && arch === 'x64') {
    // Try to detect if we're on musl (Alpine) or glibc
    try {
      execSync('ldd --version 2>&1 | grep -i musl', { stdio: 'pipe' })
      return 'linux-musl'
    } catch {
      return 'linux-glibc'
    }
  }

  throw new Error(`Unsupported platform: ${platform}-${arch}`)
}

// Download file from URL
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest)

    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Handle redirect
          return download(response.headers.location, dest).then(resolve).catch(reject)
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: ${response.statusCode}`))
          return
        }

        response.pipe(file)

        file.on('finish', () => {
          file.close()
          resolve()
        })
      })
      .on('error', (err) => {
        fs.unlink(dest, () => {}) // Delete the file on error
        reject(err)
      })
  })
}

async function install() {
  try {
    const platform = getPlatform()
    const { version } = package
    const binaryName = `bagsakan-${platform}`

    console.log(`Installing bagsakan ${version} for ${platform}...`)

    // Create bin directory
    const binDir = path.join(__dirname, '..', 'bin')
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true })
    }

    const binaryPath = path.join(binDir, 'bagsakan')

    // Construct download URL
    const repo = package.repository.url.match(/github\.com[:/](.+?)\.git/)[1]
    const downloadUrl = `https://github.com/${repo}/releases/download/v${version}/${binaryName}`

    console.log(`Downloading from ${downloadUrl}...`)

    // Download the binary
    await download(downloadUrl, binaryPath)

    // Make it executable
    fs.chmodSync(binaryPath, '755')

    console.log('✓ bagsakan installed successfully!')
    console.log(`Binary location: ${binaryPath}`)
  } catch (error) {
    console.error('Failed to install bagsakan:', error.message)
    console.error('\nYou can manually download the binary from:')
    console.error(
      `https://github.com/${package.repository.url.match(/github\.com[:/](.+?)\.git/)[1]}/releases`,
    )
    process.exit(1)
  }
}

// Run installation
install()
