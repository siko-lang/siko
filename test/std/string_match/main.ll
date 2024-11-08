@.str_1 = private unnamed_addr constant [3 x i8] c"bar", align 1
@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
@.str_2 = private unnamed_addr constant [4 x i8] c"zorp", align 1
%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_8_2 = alloca %struct.siko_Tuple_, align 4
   %i_8_1 = alloca %struct.Bool_Bool, align 4
   %i_7_2 = alloca %struct.siko_Tuple_, align 4
   %i_7_1 = alloca %struct.Bool_Bool, align 4
   %i_6_2 = alloca %struct.siko_Tuple_, align 4
   %i_6_1 = alloca %struct.Bool_Bool, align 4
   %i_5_2 = alloca %struct.siko_Tuple_, align 4
   %i_5_1 = alloca %struct.Bool_Bool, align 4
   %i_4_2 = alloca %struct.Bool_Bool, align 4
   %i_4_1 = alloca %struct.String_String, align 8
   %i_3_2 = alloca %struct.Bool_Bool, align 4
   %i_3_1 = alloca %struct.String_String, align 8
   %i_2_2 = alloca %struct.Bool_Bool, align 4
   %i_2_1 = alloca %struct.String_String, align 8
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.String_String, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.String_String, ptr %i_0_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_i_0_1_1, align 8
   %tmp_i_0_1_2 = getelementptr inbounds %struct.String_String, ptr %i_0_1, i32 0, i32 1
   store i64 3, ptr %tmp_i_0_1_2, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_1, i64 0, i1 false)
   ret void
block2:
   %tmp_i_2_1_1 = getelementptr inbounds %struct.String_String, ptr %i_2_1, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_i_2_1_1, align 8
   %tmp_i_2_1_2 = getelementptr inbounds %struct.String_String, ptr %i_2_1, i32 0, i32 1
   store i64 3, ptr %tmp_i_2_1_2, align 8
   call void @String_String_eq(ptr %i_0_1, ptr %i_2_1, ptr %i_2_2)
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_2_2, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block6
]

block3:
   %tmp_i_3_1_1 = getelementptr inbounds %struct.String_String, ptr %i_3_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_i_3_1_1, align 8
   %tmp_i_3_1_2 = getelementptr inbounds %struct.String_String, ptr %i_3_1, i32 0, i32 1
   store i64 3, ptr %tmp_i_3_1_2, align 8
   call void @String_String_eq(ptr %i_0_1, ptr %i_3_1, ptr %i_3_2)
   %tmp_switch_var_block3_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_3_2, i32 0, i32 0
   %tmp_switch_var_block3_2 = load i32, ptr %tmp_switch_var_block3_1, align 4
   switch i32 %tmp_switch_var_block3_2, label %block4 [
i32 1, label %block7
]

block4:
   %tmp_i_4_1_1 = getelementptr inbounds %struct.String_String, ptr %i_4_1, i32 0, i32 0
   store ptr @.str_2, ptr %tmp_i_4_1_1, align 8
   %tmp_i_4_1_2 = getelementptr inbounds %struct.String_String, ptr %i_4_1, i32 0, i32 1
   store i64 4, ptr %tmp_i_4_1_2, align 8
   call void @String_String_eq(ptr %i_0_1, ptr %i_4_1, ptr %i_4_2)
   %tmp_switch_var_block4_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_4_2, i32 0, i32 0
   %tmp_switch_var_block4_2 = load i32, ptr %tmp_switch_var_block4_1, align 4
   switch i32 %tmp_switch_var_block4_2, label %block5 [
i32 1, label %block8
]

block5:
   call void @Bool_Bool_False(ptr %i_5_1)
   call void @Std_Basic_Util_assert(ptr %i_5_1, ptr %i_5_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_5_2, i64 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %i_6_1)
   call void @Std_Basic_Util_assert(ptr %i_6_1, ptr %i_6_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_6_2, i64 0, i1 false)
   br label %block1
block7:
   call void @Bool_Bool_True(ptr %i_7_1)
   call void @Std_Basic_Util_assert(ptr %i_7_1, ptr %i_7_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_7_2, i64 0, i1 false)
   br label %block1
block8:
   call void @Bool_Bool_False(ptr %i_8_1)
   call void @Std_Basic_Util_assert(ptr %i_8_1, ptr %i_8_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_8_2, i64 0, i1 false)
   br label %block1
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %i_6_1 = alloca %struct.siko_Tuple_, align 4
   %i_4_2 = alloca %struct.siko_Tuple_, align 4
   %i_4_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_0_1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %i_4_1)
   call void @siko_Tuple_(ptr %i_4_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_4_2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %i_6_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_6_1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_False*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


