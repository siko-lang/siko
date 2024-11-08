%struct.Main_Bool = type { i32, [0 x i8] }

%struct.Main_Bool_False = type {  }

%struct.Main_Bool_True = type {  }

%struct.Main_Container_Main_Bool = type { %struct.Main_Bool }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Container_Main_Bool(ptr noundef %item, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Container_Main_Bool, align 4
   %field0 = getelementptr inbounds %struct.Main_Container_Main_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %item, ptr align 4 %field0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_0_7 = alloca %struct.siko_Tuple_, align 4
   %b_1 = alloca %struct.Main_Bool, align 4
   %i_0_5 = alloca %struct.Main_Bool, align 4
   %i_0_4 = alloca %struct.Main_Container_Main_Bool, align 4
   %a_0 = alloca %struct.Main_Container_Main_Bool, align 4
   %i_0_2 = alloca %struct.Main_Container_Main_Bool, align 4
   %i_0_1 = alloca %struct.Main_Bool, align 4
   call void @Main_Bool_True(ptr %i_0_1)
   call void @Main_Container_Main_Bool(ptr %i_0_1, ptr %i_0_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %i_0_2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_4, ptr align 4 %a_0, i64 4, i1 false)
   call void @Main_other_Main_Bool(ptr %i_0_4, ptr %i_0_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b_1, ptr align 4 %i_0_5, i64 4, i1 false)
   call void @siko_Tuple_(ptr %i_0_7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_7, i64 0, i1 false)
   ret void
}

define private void @Main_other_Main_Bool(ptr noundef %c, ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.Main_Container_Main_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_1, ptr align 4 %c, i64 4, i1 false)
   %i_0_2 = getelementptr inbounds %struct.Main_Container_Main_Bool, ptr %i_0_1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_2, i64 4, i1 false)
   ret void
}

define private void @Main_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Bool, align 4
   %tag = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


