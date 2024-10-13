%struct.Int_Int = type { i64 }

define %struct.Int_Int @Int_Int() {
   %1 = alloca i64, align 8
   ret i64 %1
}

define void @Main_foo() {
   ret void
}

define void @Main_main() {
   ret void
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


