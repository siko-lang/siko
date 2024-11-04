%struct.Main_Bool = type { i32, [0 x i8] }

%struct.Main_Bool_False = type {  }

%struct.Main_Bool_True = type {  }

%struct.Main_FooBar = type { i32, [8 x i8] }

%struct.Main_FooBar_Bar = type { %struct.Main_Bool, %struct.Main_Bool }

%struct.Main_FooBar_Foo = type { %struct.Main_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_Bool = type { %struct.Main_Bool }

%struct.siko_Tuple_Main_Bool__Main_Bool = type { %struct.Main_Bool, %struct.Main_Bool }

define void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @siko_Tuple_Main_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @siko_Tuple_Main_Bool__Main_Bool(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_Bool__Main_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Main_Bool__Main_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f1, ptr align 4 %field1, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 8, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_0_6 = alloca %struct.siko_Tuple_, align 4
   %i_0_5 = alloca %struct.Main_FooBar, align 4
   %i_0_4 = alloca %struct.Main_Bool, align 4
   %i_0_3 = alloca %struct.Main_Bool, align 4
   %i_0_2 = alloca %struct.Main_FooBar, align 4
   %i_0_1 = alloca %struct.Main_Bool, align 4
   call void @Main_Bool_True(ptr %i_0_1)
   call void @Main_FooBar_Foo(ptr %i_0_1, ptr %i_0_2)
   call void @Main_Bool_True(ptr %i_0_3)
   call void @Main_Bool_True(ptr %i_0_4)
   call void @Main_FooBar_Bar(ptr %i_0_3, ptr %i_0_4, ptr %i_0_5)
   call void @siko_Tuple_(ptr %i_0_6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_6, i8 0, i1 false)
   ret void
}

define void @Main_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Bool, align 4
   %tag = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @Main_FooBar_Bar(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_FooBar_Bar*
   %field0 = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %payload2, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   %field1 = getelementptr inbounds %struct.Main_FooBar_Bar, ptr %payload2, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f1, ptr align 4 %field1, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 12, i1 false)
   ret void
}

define void @Main_FooBar_Foo(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_FooBar, align 4
   %tag = getelementptr inbounds %struct.Main_FooBar, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_FooBar, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_FooBar_Foo*
   %field0 = getelementptr inbounds %struct.Main_FooBar_Foo, ptr %payload2, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 12, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


