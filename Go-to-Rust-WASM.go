package main

import (
	"encoding/binary"
	"fmt"
	"log"
	"os"

	"github.com/bytecodealliance/wasmtime-go/v26"
)

func main() {
	engine := wasmtime.NewEngine()
	store := wasmtime.NewStore(engine)
	context := store.Context()

	wasmBytes, err := os.ReadFile("validator.wasm")
	if err != nil {
		log.Fatalf("Failed to read WASM file: %v", err)
	}

	module, err := wasmtime.NewModule(engine, wasmBytes)
	if err != nil {
		log.Fatalf("Failed to compile module: %v", err)
	}

	instance, err := wasmtime.NewInstance(context, module, []wasmtime.AsExtern{})
	if err != nil {
		log.Fatalf("Failed to instantiate module: %v", err)
	}

	// 1. Get references to exported functions and memory
	allocFn := instance.GetExport(context, "alloc").Func()
	processFn := instance.GetExport(context, "process_string").Func()
	memory := instance.GetExport(context, "memory").Memory()

	// 2. Prepare the string payload
	inputString := "Hello from Go to Rust-WASM!"
	inputBytes := []byte(inputString)
	inputLen := len(inputBytes)

	// 3. Call Rust's `alloc` to get a pointer to a buffer inside WASM memory
	allocResult, err := allocFn.Call(context, inputLen)
	if err != nil {
		log.Fatalf("Allocation failed: %v", err)
	}
	ptr := int(allocResult.(int32))

	// 4. Write Go string bytes directly into Wasmtime memory space
	memData := memory.Data(context)
	copy(memData[ptr:ptr+inputLen], inputBytes)

	// 5. Call `process_string` passing pointer and length
	processResult, err := processFn.Call(context, ptr, inputLen)
	if err != nil {
		log.Fatalf("Processing failed: %v", err)
	}
	resultPtr := int(processResult.(int32))

	// 6. Read back the response from WASM memory
	// (Reading the 4-byte length prefix we structured in Rust)
	outputLen := int(binary.LittleEndian.Uint32(memData[resultPtr : resultPtr+4]))
	outputBytes := memData[resultPtr+4 : resultPtr+4+outputLen]

	fmt.Printf("Result from WASM: %s\n", string(outputBytes))
}
