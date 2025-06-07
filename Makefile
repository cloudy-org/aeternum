ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    detected_os := Windows
else
    detected_os := $(shell uname)  # same as "uname -s"
endif

build:
	cargo build --release

install: install-shortcut
ifeq ($(detected_os), Windows)
	copy ".\target\release\aeternum.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./target/release/aeternum /usr/bin/
endif

install-shortcut:
ifeq ($(detected_os), Windows)
	echo "Not implemented!"
else
	sudo cp ./assets/sparkles.png /usr/share/icons/aeternum.png
	sudo cp ./assets/aeternum.desktop /usr/share/applications/
	sudo update-desktop-database /usr/share/applications/
endif

uninstall: uninstall-shortcut
ifeq ($(detected_os), Windows)
	del "$(USERPROFILE)\.cargo\bin\aeternum.exe"
else
	sudo rm /usr/bin/aeternum
endif

uninstall-shortcut:
ifeq ($(detected_os), Windows)
	echo "Not implemented!"
else
	sudo rm /usr/share/icons/aeternum.png
	sudo rm /usr/share/applications/aeternum.desktop
	sudo update-desktop-database /usr/share/applications/
endif

clean:
	cargo clean

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote