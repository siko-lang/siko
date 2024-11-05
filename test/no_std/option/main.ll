%struct.Main_Bool = type { i32, [0 x i8] }

%struct.Main_Bool_False = type {  }

%struct.Main_Bool_True = type {  }

%struct.Main_Option_Main_Bool = type { i32, [4 x i8] }

%struct.Main_Option_None_Main_Bool = type {  }

%struct.Main_Option_Some_Main_Bool = type { %struct.Main_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_Bool = type { %struct.Main_Bool }

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

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_12_2 = alloca %struct.Main_Bool, align 4
   %a_5 = alloca %struct.Main_Bool, align 4
   %i_10_1 = alloca %struct.Main_Bool, align 4
   %i_7_2 = alloca %struct.siko_Tuple_, align 4
   %i_7_1 = alloca %struct.Main_Bool, align 4
   %i_6_2 = alloca %struct.Main_Bool, align 4
   %a_2 = alloca %struct.Main_Bool, align 4
   %i_4_1 = alloca %struct.Main_Bool, align 4
   %match_var_4 = alloca %struct.siko_Tuple_, align 4
   %i_1_4 = alloca %struct.Main_Option_Main_Bool, align 4
   %a_3 = alloca %struct.Main_Option_Main_Bool, align 4
   %i_1_2 = alloca %struct.Main_Option_Main_Bool, align 4
   %i_1_1 = alloca %struct.Main_Bool, align 4
   %match_var_1 = alloca %struct.siko_Tuple_, align 4
   %i_0_4 = alloca %struct.Main_Option_Main_Bool, align 4
   %a_0 = alloca %struct.Main_Option_Main_Bool, align 4
   %i_0_2 = alloca %struct.Main_Option_Main_Bool, align 4
   %i_0_1 = alloca %struct.Main_Bool, align 4
   call void @Main_Bool_True(ptr %i_0_1)
   call void @Main_Option_Some_Main_Bool(ptr %i_0_1, ptr %i_0_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %i_0_2, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_4, ptr align 4 %a_0, i8 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_1, i8 4, i1 false)
   call void @Main_Option_None_Main_Bool(ptr %i_1_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_3, ptr align 4 %i_1_2, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_4, ptr align 4 %a_3, i8 8, i1 false)
   br label %block8
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %i_0_4, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   %i_3_1 = bitcast %struct.Main_Option_Main_Bool* %i_0_4 to %struct.siko_Tuple_*
   br label %block4
block4:
   call void @Main_Bool_False(ptr %i_4_1)
   br label %block1
block5:
   %i_5_1 = bitcast %struct.Main_Option_Main_Bool* %i_0_4 to %struct.siko_Tuple_Main_Bool*
   %i_5_2 = getelementptr inbounds %struct.siko_Tuple_Main_Bool, ptr %i_5_1, i32 0, i32 0
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_2, ptr align 4 %i_5_2, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_6_2, ptr align 4 %a_2, i8 4, i1 false)
   br label %block1
block7:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_7_1, ptr align 4 %match_var_4, i8 4, i1 false)
   call void @siko_Tuple_(ptr %i_7_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_7_2, i8 0, i1 false)
   ret void
block8:
   %tmp_switch_var_block8_1 = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %i_1_4, i32 0, i32 0
   %tmp_switch_var_block8_2 = load i32, ptr %tmp_switch_var_block8_1, align 4
   switch i32 %tmp_switch_var_block8_2, label %block9 [
i32 1, label %block11
]

block9:
   %i_9_1 = bitcast %struct.Main_Option_Main_Bool* %i_1_4 to %struct.siko_Tuple_*
   br label %block10
block10:
   call void @Main_Bool_False(ptr %i_10_1)
   br label %block7
block11:
   %i_11_1 = bitcast %struct.Main_Option_Main_Bool* %i_1_4 to %struct.siko_Tuple_Main_Bool*
   %i_11_2 = getelementptr inbounds %struct.siko_Tuple_Main_Bool, ptr %i_11_1, i32 0, i32 0
   br label %block12
block12:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_5, ptr align 4 %i_11_2, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_12_2, ptr align 4 %a_5, i8 4, i1 false)
   br label %block7
}

define void @Main_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Bool, align 4
   %tag = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Bool_False*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
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

define void @Main_Option_None_Main_Bool(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Option_Main_Bool, align 4
   %tag = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Option_None_Main_Bool*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 8, i1 false)
   ret void
}

define void @Main_Option_Some_Main_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Option_Main_Bool, align 4
   %tag = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Option_Main_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Option_Some_Main_Bool*
   %field0 = getelementptr inbounds %struct.Main_Option_Some_Main_Bool, ptr %payload2, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


