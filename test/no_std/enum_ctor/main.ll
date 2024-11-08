%struct.Main_Bool = type { i32, [0 x i8] }

%struct.Main_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Main_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_FooBar = type { i32, [8 x i8] }

%struct.Main_FooBar_Bar = type { i32, %struct.siko_Tuple_Main_Bool__Main_Bool }

%struct.Main_FooBar_Foo = type { i32, %struct.siko_Tuple_Main_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_Bool = type { %struct.Main_Bool }

%struct.siko_Tuple_Main_Bool__Main_Bool = type { %struct.Main_Bool, %struct.Main_Bool }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Main_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Main_Bool__Main_Bool(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_Bool__Main_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field1, ptr align 4 %f1, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i6 = alloca %struct.siko_Tuple_, align 4
   %b0i5 = alloca %struct.Main_FooBar, align 4
   %b0i4 = alloca %struct.Main_Bool, align 4
   %b0i3 = alloca %struct.Main_Bool, align 4
   %b0i2 = alloca %struct.Main_FooBar, align 4
   %b0i1 = alloca %struct.Main_Bool, align 4
   call void @Main_Bool_True(ptr %b0i1)
   call void @Main_FooBar_Foo(ptr %b0i1, ptr %b0i2)
   call void @Main_Bool_True(ptr %b0i3)
   call void @Main_Bool_True(ptr %b0i4)
   call void @Main_FooBar_Bar(ptr %b0i3, ptr %b0i4, ptr %b0i5)
   call void @siko_Tuple_(ptr %b0i6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i6, i64 0, i1 false)
   ret void
}

define private void @Main_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Main_Bool_True, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_FooBar_Bar(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar_Bar, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %payload1, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field1, ptr align 4 %f1, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 12, i1 false)
   ret void
}

define private void @Main_FooBar_Foo(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar_Foo, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar_Foo, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar_Foo, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 12, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


