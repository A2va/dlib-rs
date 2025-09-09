option("simd", { description = "Set simd configs to dlib"})


-- disable blas on windows as there is only a dynamic lib distribution
-- and that phony target do not seem to install dlls
local features = {blas = not is_plat("windows"), jpg = true, png = true}
if get_config("simd") then
    features.simd = get_config("simd")
end

add_requires("dlib v19.24.9", {configs = features})

target("dlib")
    set_kind("phony")
    add_packages("dlib", {public = true})
