%struct.Main_SomeClass = type {  }

%struct.Main_SomeEnum = type { i32, [0 x i32] }

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
   %unit_13 = alloca %struct.siko_Tuple_, align 4
   %call_12 = alloca %struct.siko_Tuple_, align 4
   %valueRef_10 = alloca %struct.Main_SomeEnum, align 4
   %e_9 = alloca %struct.Main_SomeEnum, align 4
   %call_8 = alloca %struct.Main_SomeEnum, align 4
   %call_7 = alloca %struct.siko_Tuple_, align 4
   %call_6 = alloca %struct.siko_Tuple_, align 4
   %valueRef_4 = alloca %struct.Main_SomeClass, align 4
   %call_3 = alloca %struct.siko_Tuple_, align 4
   %c_2 = alloca %struct.Main_SomeClass, align 4
   %call_1 = alloca %struct.Main_SomeClass, align 4
   call void @Main_SomeClass(ptr %call_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %c_2, ptr align 4 %call_1, i64 0, i1 false)
   call void @Main_SomeClass_foo(ptr %call_3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_4, ptr align 4 %c_2, i64 0, i1 false)
   call void @Main_SomeClass_foo2(ptr %valueRef_4, ptr %call_6)
   call void @Main_SomeEnum_foo(ptr %call_7)
   call void @Main_SomeEnum_Foo(ptr %call_8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %e_9, ptr align 4 %call_8, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_10, ptr align 4 %e_9, i64 4, i1 false)
   call void @Main_SomeEnum_foo2(ptr %valueRef_10, ptr %call_12)
   call void @siko_Tuple_(ptr %unit_13)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_13, i64 0, i1 false)
   ret void
}

define private void @Main_SomeClass_foo(ptr noundef %fn_result) {
block0:
   %unit_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %unit_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeClass_foo2(ptr noundef %self, ptr noundef %fn_result) {
block0:
   %unit_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %unit_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeEnum_Foo(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_SomeEnum_Foo, align 4
   %tag = getelementptr inbounds %struct.Main_SomeEnum_Foo, ptr %this, i32 0, i32 0
   store volatile i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_SomeEnum_Foo, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_SomeEnum_foo(ptr noundef %fn_result) {
block0:
   %unit_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %unit_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_1, i64 0, i1 false)
   ret void
}

define private void @Main_SomeEnum_foo2(ptr noundef %self, ptr noundef %fn_result) {
block0:
   %unit_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %unit_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_1, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


