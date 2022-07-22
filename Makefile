GLSLC=$(VULKAN_SDK)/macOS/bin/glslc -O -std=450 --target-env=vulkan1.3

SRC					:= crates/lambda_internal/src

ifeq ($(OS),Windows_NT)
SOURCEDIRS	:= $(SRC)
FIXPATH = $(subst /,\,$1)
RM			:= del /q /f
MD	:= mkdir
else
SOURCEDIRS	:= $(shell find $(SRC) -type d)
FIXPATH = $1
RM = rm -f
MD	:= mkdir -p
endif

FRAG_SHADERS		:= $(wildcard $(patsubst %,%/*.frag, $(SOURCEDIRS)))
VERT_SHADERS		:= $(wildcard $(patsubst %,%/*.vert, $(SOURCEDIRS)))

SHADER_FOLDERS 		:= $(shell ls ${SRC}/shaders)

clean_shaders:
	$(RM) $(wildcard $(patsubst %,%/*.spv, $(SOURCEDIRS)))

compile_shaders: clean_shaders
	for texture_type in $(SHADER_FOLDERS) ; do \
		GLSLC $(call FIXPATH,$(SRC)/shaders/$$texture_type/shader.vert) -o $(call FIXPATH,$(SRC)/shaders/$$texture_type/vert.spv) ; \
		GLSLC $(call FIXPATH,$(SRC)/shaders/$$texture_type/shader.frag) -o $(call FIXPATH,$(SRC)/shaders/$$texture_type/frag.spv) ; \
	done
