@echo off

rem Check if both output and rotation are provided as arguments
if "%~2"=="" (
    echo Usage: %~nx0 out_file rotation
    exit /b 1
)

set "out_folder=.\temp\debug"
set "in_file=%out_folder%\0deg.mp4"
set "out_file=%out_folder%\%~1"
set "rotation=%~2"
set /a "inv_rotation=rotation * -1"
set "temp_file=%out_folder%\temp.mp4"

rem First we create a temp video that's actually rotated and is not just side data
ffmpeg -display_rotation %rotation% -i %in_file% %temp_file%

rem Then we create the actual video with side data from the temp rotated video
ffmpeg -display_rotation %inv_rotation% -i %temp_file% -codec copy -y %out_file%

rem Delete the temp video
del %temp_file%

echo Video "%out_file%" with displaymatrix=%rotation%deg created successfully
