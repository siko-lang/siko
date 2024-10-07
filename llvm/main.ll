define i32 @main() {
entry:
  %x = alloca i32, align 4
  store i32 42, i32* %x
  %loaded_value = load i32, i32* %x
  ret i32 %loaded_value
}