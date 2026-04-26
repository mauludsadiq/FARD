IMAGE = fard-bootstrap

vm: vm.asm
	docker run --platform linux/amd64 --rm -v $(PWD):/work $(IMAGE) \
		sh -c "nasm -f elf64 -o vm.o vm.asm && ld -o fardvm vm.o"

test: vm
	docker run --platform linux/amd64 --rm -v $(PWD):/work $(IMAGE) \
		./fardvm test.fardc

clean:
	rm -f vm.o fardvm

.PHONY: vm test clean
