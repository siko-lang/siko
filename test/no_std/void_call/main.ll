define void @Main_foo() {
ret void
}

define void @Main_main() {
call void @Main_foo()
ret void
}

define i32 @main() {
call void @Main_main()
ret i32 0
}


