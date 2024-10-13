%struct.Int_Int = type { i64 }

%struct.Main_Struct1 = type { i64 }

define i64 @Int_Int() {
   %1 = alloca i64, align 8
   %2 = load i64, ptr %1, align 8
   ret i64 %2
}

define %struct.Main_Struct1 @Main_Struct1(i64 %num) {
   %1 = alloca i64, align 8
   %2 = load i64, ptr %1, align 8
   ret i64 %2
}

define i64 @Main_foo() {
   %1 = call i64 @Int_Int()
   ret i64 %1
}

define void @Main_foo2() {
   %1 = call i64 @Int_Int()
   %2 = call %struct.Main_Struct1 @Main_Struct1(%1)
   ret void
}

define void @Main_main() {
   %1 = call i64 @Main_foo()
   call void @Main_foo2()
   ret void
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


