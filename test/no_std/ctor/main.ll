%struct.Int_Int = type { i32 }

define %struct.Int_Int @Int_Int() {
   %1 = alloca i32, align 4
   %2 = load %struct.Int_Int, ptr %1, align 4
   ret %struct.Int_Int %2
}

define void @Main_foo() {
   call void @Int_Int()
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


