define void @Main_main() {
ret void
}

define i32 @main() {
call void @Main_main()
ret i32 0
}


