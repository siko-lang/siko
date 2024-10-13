%struct.Int_Int = type { i64 }

define i64 @Int_Int() {
   %1 = alloca i64, align 8
   %2 = load i64, ptr %1, align 8
   ret i64 %2
}

define i64 @Main_foo() {
   %1 = call i64 @Int_Int()
   ret i64 %1
}

define void @Main_main() {
   %1 = call i64 @Main_foo()
   ret void
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


