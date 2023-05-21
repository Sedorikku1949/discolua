local targets = { "luajit", "lua54", "lua53", "lua52", "lua51" }

-- ensuring that the folder "builds" exists
os.execute("mkdir -p builds")

for _, target in ipairs(targets) do
    print("Building for target: " .. target)
    -- Building the library
    os.execute("cargo build --release --features " .. target)

    print("Copying the library to the build folder")

    -- Copying the library to the builds folder in a folder with the target name
    -- With the target in the name
    os.execute("mkdir -p builds/" .. target)
    os.execute("cp target/release/libdiscolua.so builds/" .. target .. "/libdiscolua.so")

    print("Building for target: " .. target .. " done!")
end