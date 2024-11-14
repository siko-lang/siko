%struct.Bool_Bool = type { i32, [0 x i32] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_FooBar = type { i32, [2 x i32] }

%struct.Main_FooBar_Bar = type { i32, %struct.siko_Tuple_Bool_Bool__Bool_Bool }

%struct.Main_FooBar_Foo = type { i32, %struct.siko_Tuple_Bool_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool = type { %struct.Bool_Bool }

%struct.siko_Tuple_Bool_Bool__Bool_Bool = type { %struct.Bool_Bool, %struct.Bool_Bool }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool__Bool_Bool(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool__Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field1, ptr align 4 %f1, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_6 = alloca %struct.siko_Tuple_, align 4
   %call_5 = alloca %struct.Main_FooBar, align 4
   %call_4 = alloca %struct.Bool_Bool, align 4
   %call_3 = alloca %struct.Bool_Bool, align 4
   %call_2 = alloca %struct.Main_FooBar, align 4
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %call_1)
   call void @Main_FooBar_Foo(ptr %call_1, ptr %call_2)
   call void @Bool_Bool_True(ptr %call_3)
   call void @Bool_Bool_True(ptr %call_4)
   call void @Main_FooBar_Bar(ptr %call_3, ptr %call_4, ptr %call_5)
   call void @siko_Tuple_(ptr %unit_6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_6, i64 0, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store volatile i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_FooBar_Bar(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar_Bar, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %this, i32 0, i32 0
   store volatile i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %payload1, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field1, ptr align 4 %f1, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 12, i1 false)
   ret void
}

define private void @Main_FooBar_Foo(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar_Foo, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar_Foo, ptr %this, i32 0, i32 0
   store volatile i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar_Foo, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 12, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


