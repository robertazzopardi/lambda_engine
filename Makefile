OS_NAME := $(shell uname -s | tr A-Z a-z)

GLSLC=glslc
GLSLC_FLAGS=-O -std=450 --target-env=vulkan1.3

SRC							:= crates/wave_internal/src

ifeq ($(OS),Windows_NT)
SOURCEDIRS					:= $(SRC)
FIXPATH = $(subst /,\,$1)
RM							:= del /q /f
MD							:= mkdir
else
SOURCEDIRS					:= $(shell find $(SRC) -type d)
FIXPATH = $1
RM = rm -f
MD							:= mkdir -p
endif

FRAG_SHADERS				:= $(wildcard $(patsubst %,%/*.frag, $(SOURCEDIRS)))
VERT_SHADERS				:= $(wildcard $(patsubst %,%/*.vert, $(SOURCEDIRS)))

SHADER_FOLDERS 				:= $(shell ls ${SRC}/shaders)

clean_shaders:
	$(RM) $(wildcard $(patsubst %,%/*.spv, $(SOURCEDIRS)))

compile_shaders: clean_shaders
	for texture_type in $(SHADER_FOLDERS) ; do \
		$(GLSLC) $(GLSLC_FLAGS) $(call FIXPATH,$(SRC)/shaders/$$texture_type/shader.vert) -o $(call FIXPATH,$(SRC)/shaders/$$texture_type/vert.spv) ; \
		$(GLSLC) $(GLSLC_FLAGS) $(call FIXPATH,$(SRC)/shaders/$$texture_type/shader.frag) -o $(call FIXPATH,$(SRC)/shaders/$$texture_type/frag.spv) ; \
	done

check:
	cargo clippy -- \
	    -Dwarnings \
            -Dclippy::pedantic \
            -Dclippy::nursery \
            -Dclippy::correctness \
            -Dclippy::complexity \
            -Dclippy::perf 
