import { writeFileSync } from 'fs'
import { compileFromFile } from 'json-schema-to-typescript'

// compile from file
compileFromFile('rust_types.json',{format:false})
  .then(ts => writeFileSync('src/rustTypes.ts', ts))