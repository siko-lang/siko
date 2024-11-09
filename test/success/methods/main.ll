%struct.Main_SomeClass = type {  }

%struct.Main_SomeEnum = type { i32, [0 x i8] }

%struct.Main_SomeEnum_Foo = type { i32, %struct.siko_Tuple_ }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_SomeClass(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_SomeClass, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i13 = alloca %struct.siko_Tuple_, align 4
   %b0i12 = alloca %struct.siko_Tuple_, align 4
   %b0i10 = alloca %struct.Main_SomeEnum, align 4
   %e_1 = alloca %struct.Main_SomeEnum, align 4
   %b0i8 = alloca %struct.Main_SomeEnum, align 4
   %b0i7 = alloca %struct.siko_Tuple_, align 4
   %b0i6 = alloca %struct.siko_Tuple_, align 4
   %b0i4 = alloca %struct.Main_SomeClass, align 4
   %b0i3 = alloca %struct.siko_Tuple_, align 4
   %c_0 = alloca %struct.Main_SomeClass, align 4
   %b0i1 = alloca %struct.Main_SomeClass, align 4
   call void @Main_SomeClass(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %c_0, ptr align 4 %b0i1, i64 0, i1 false)
   call void @Main_SomeClass_foo(ptr %b0i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i4, ptr align 4 %c_0, i64 0, i1 false)
   call void @Main_SomeClass_foo2(ptr %b0i4, ptr %b0i6)
   call void @Main_SomeEnum_foo(ptr %b0i7)
   call void @Main_SomeEnum_Foo(ptr %b0i8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %e_1, ptr align 4 %b0i8, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i10, ptr align 4 %e_1, i64 4, i1 false)
   call void @Main_SomeEnum_foo2(ptr %b0i10, ptr %b0i12)
   call void @siko_Tuple_(ptr %b0i13)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i13, i64 0, i1 false)
   ret void
}

define private void @Main_SomeClass_foo(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeClass_foo2(ptr noundef %self, ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeEnum_Foo(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_SomeEnum_Foo, align 4
   %tag = getelementptr inbounds %struct.Main_SomeEnum_Foo, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_SomeEnum_Foo, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_SomeEnum_foo(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeEnum_foo2(ptr noundef %self, ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


