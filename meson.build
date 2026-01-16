project(
  'vxectl',
  'cpp',
  meson_version : '>= 1.3.0',
  version : '0.1',
  default_options : ['warning_level=3'],
)

dependencies = [
  dependency('hidapi', required : true),
  dependency('CLI11', required : true),
]

source_files = files('src/vxectl.cpp', 'src/dpi_stage.cpp', 'src/set.cpp')

executable(
  'vxectl',
  source_files,
  dependencies : dependencies,
)