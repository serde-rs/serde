$ErrorActionPreference = 'Stop'

$files = @(
  'serde_core/Cargo.toml',
  'serde_derive/build.rs',
  'serde_derive/Cargo.toml',
  'serde_derive/src/de.rs',
  'serde_derive/src/dummy.rs',
  'serde_derive/src/internals/attr.rs',
  'serde_derive/src/lib.rs',
  'serde_derive/src/ser.rs',
  'serde_state/src/export.rs',
  'serde_state/src/private/de.rs',
  'serde_state/src/private/ser.rs',
  'test_suite/Cargo.toml',
  'test_suite/no_std/Cargo.toml',
  'test_suite/no_std/src/main.rs',
  'test_suite/tests/test_annotations.rs',
  'test_suite/tests/test_gen.rs'
)

foreach ($f in $files) {
  if (-not (Test-Path -LiteralPath $f)) {
    continue
  }

  $lines = Get-Content -LiteralPath $f
  $out = New-Object 'System.Collections.Generic.List[string]'
  $state = 0

  foreach ($line in $lines) {
    if ($line -like '<<<<<<<*') {
      $state = 1
      continue
    }
    elseif ($line -like '=======*') {
      if ($state -eq 1) {
        $state = 2
        continue
      }
    }
    elseif ($line -like '>>>>>>>*') {
      if ($state -eq 2) {
        $state = 0
        continue
      }
    }

    if ($state -eq 0 -or $state -eq 1) {
      $out.Add($line)
    }
  }

  Set-Content -LiteralPath $f -Value $out -Encoding utf8
}

git rm -r -f --ignore-unmatch serde serde_core serde_derive_internals
if ($LASTEXITCODE -ne 0) {
  throw 'git rm failed'
}

git add -A
if ($LASTEXITCODE -ne 0) {
  throw 'git add failed'
}

Write-Host 'Marker cleanup complete.'
