#!/usr/bin/env node
//const sh = require('shelljs') //LMT remove dependencies for now, simplify. only core node modules
const child_process=require("child_process")
const path=require("path")

//sh.fatal = true // same as "set -e"

//sh.cd(__dirname)

// Note: see flags in ./cargo/config
//sh.exec('cargo build --target wasm32-unknown-unknown --release')
child_process.execSync('cargo build --target wasm32-unknown-unknown --release',{ stdio: 'inherit' })

/*
const outdir = '../../out'
execSync('mkdir -p ../../out')

sh.ls('./target/wasm32-unknown-unknown/release/*.wasm').map(src => {
  const output = path.basename(src)
    .replace('.wasm', '-rs.wasm')
    .replace(/_/g, '-')

  console.log(`\ncopying [ ${src} ] to [ out/${output} ]`);

  sh.cp(src, `${outdir}/${output}`)
})
*/
