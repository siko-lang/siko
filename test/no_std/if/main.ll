%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %i_0_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_1, i8 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_2_2 = alloca %struct.siko_Tuple_, align 4
   %i_2_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_3 = alloca %struct.siko_Tuple_, align 4
   %i_1_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %if_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %i_0_1)
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %if_var_0, i8 0, i1 false)
   call void @Main_foo(ptr %i_1_2)
   call void @siko_Tuple_(ptr %i_1_3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_3, i8 0, i1 false)
   ret void
block2:
   call void @Main_foo(ptr %i_2_1)
   call void @siko_Tuple_(ptr %i_2_2)
   br label %block1
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


